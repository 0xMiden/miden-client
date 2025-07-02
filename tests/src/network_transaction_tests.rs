use std::{sync::Arc, vec};

use miden_client::{
    Felt, Word, ZERO,
    account::{Account, AccountBuilder, AccountStorageMode, StorageSlot},
    note::NoteTag,
    testing::{
        common::{
            TestClient, create_test_client, execute_tx_and_sync, insert_new_wallet,
            wait_for_blocks, wait_for_tx,
        },
        note::NoteBuilder,
    },
    transaction::{OutputNote, TransactionRequestBuilder, TransactionScript},
};
use miden_lib::transaction::TransactionKernel;
use miden_objects::{
    Digest,
    account::AccountComponent,
    assembly::{Assembler, DefaultSourceManager, Library, LibraryPath, Module, ModuleKind},
};

// HELPERS
// ================================================================================================

const COUNTER_CONTRACT: &str = "
        use.miden::account
        use.std::sys

        # => []
        export.get_count
            push.0
            exec.account::get_item
            exec.sys::truncate_stack
        end

        # => []
        export.increment_count
            push.0
            # => [index]
            exec.account::get_item
            # => [count]
            push.1 add
            # => [count+1]
            push.0
            # [index, count+1]
            exec.account::set_item
            # => []
            push.1 exec.account::incr_nonce
            # => []
            exec.sys::truncate_stack
            # => []
        end";

/// Deploys a counter contract as a network account
async fn deploy_counter_contract(
    client: &mut TestClient,
    storage_mode: AccountStorageMode,
) -> Result<(Account, Library), String> {
    let (acc, seed, library) = get_counter_contract_account(client, storage_mode).await;

    client.add_account(&acc, Some(seed), false).await.unwrap();

    let assembler: Assembler = TransactionKernel::assembler().with_debug_mode(true);
    let tx_script = TransactionScript::compile(
        "use.external_contract::counter_contract
        begin
            call.counter_contract::increment_count
        end",
        assembler.with_library(&library).unwrap(),
    )
    .unwrap();

    // Build a transaction request with the custom script
    let tx_increment_request =
        TransactionRequestBuilder::new().with_custom_script(tx_script).build().unwrap();

    // Execute the transaction locally
    let tx_result = client.new_transaction(acc.id(), tx_increment_request).await.unwrap();
    let tx_id = tx_result.executed_transaction().id();
    client.submit_transaction(tx_result).await.unwrap();
    wait_for_tx(client, tx_id).await;

    Ok((acc, library))
}

async fn get_counter_contract_account(
    client: &mut TestClient,
    storage_mode: AccountStorageMode,
) -> (Account, Word, Library) {
    let counter_component = AccountComponent::compile(
        COUNTER_CONTRACT,
        TransactionKernel::assembler(),
        vec![StorageSlot::empty_value()],
    )
    .unwrap()
    .with_supports_all_types();

    let mut init_seed = [0u8; 32];
    client.rng().fill_bytes(&mut init_seed);

    let (account, seed) = AccountBuilder::new(init_seed)
        .storage_mode(storage_mode)
        .with_component(counter_component)
        .build()
        .unwrap();

    let assembler: Assembler = TransactionKernel::assembler().with_debug_mode(true);
    let source_manager = Arc::new(DefaultSourceManager::default());
    let module = Module::parser(ModuleKind::Library)
        .parse_str(
            LibraryPath::new("external_contract::counter_contract").unwrap(),
            COUNTER_CONTRACT,
            &source_manager,
        )
        .unwrap();
    let library = assembler.clone().assemble_library([module]).unwrap();

    (account, seed, library)
}

// TESTS
// ================================================================================================

#[tokio::test]
async fn counter_contract_ntx() {
    const BUMP_NOTE_NUMBER: u64 = 5;
    let (mut client, keystore) = create_test_client().await;
    client.sync_state().await.unwrap();

    let (network_account, library) =
        deploy_counter_contract(&mut client, AccountStorageMode::Network).await.unwrap();

    assert_eq!(
        client
            .get_account(network_account.id())
            .await
            .unwrap()
            .unwrap()
            .account()
            .storage()
            .get_item(0)
            .unwrap(),
        Digest::from([ZERO, ZERO, ZERO, Felt::new(1)])
    );

    let (native_account, _native_seed, _) =
        insert_new_wallet(&mut client, AccountStorageMode::Public, &keystore)
            .await
            .unwrap();

    let assembler = TransactionKernel::assembler()
        .with_debug_mode(true)
        .with_library(library)
        .unwrap();

    let mut network_notes = vec![];

    for _ in 0..BUMP_NOTE_NUMBER {
        network_notes.push(OutputNote::Full(
            NoteBuilder::new(native_account.id(), client.rng())
                .code(
                    "use.external_contract::counter_contract
                begin
                    call.counter_contract::increment_count
                end",
                )
                .tag(NoteTag::from_account_id(network_account.id()).into())
                .build(&assembler)
                .unwrap(),
        ));
    }

    let tx_request = TransactionRequestBuilder::new()
        .with_own_output_notes(network_notes)
        .build()
        .unwrap();

    execute_tx_and_sync(&mut client, native_account.id(), tx_request).await;

    wait_for_blocks(&mut client, 2).await;

    let a = client
        .test_rpc_api()
        .get_account_details(network_account.id())
        .await
        .unwrap()
        .account()
        .cloned()
        .unwrap();

    assert_eq!(
        a.storage().get_item(0).unwrap(),
        Digest::from([ZERO, ZERO, ZERO, Felt::new(1 + BUMP_NOTE_NUMBER)])
    );
}

#[tokio::test]
async fn recall_note_before_ntx_consumes_it() {
    let (mut client, keystore) = create_test_client().await;
    client.sync_state().await.unwrap();

    let (network_account, library) =
        deploy_counter_contract(&mut client, AccountStorageMode::Network).await.unwrap();

    let native_account = deploy_counter_contract(&mut client, AccountStorageMode::Public)
        .await
        .unwrap()
        .0;

    let wallet = insert_new_wallet(&mut client, AccountStorageMode::Public, &keystore)
        .await
        .unwrap()
        .0;

    let assembler = TransactionKernel::assembler()
        .with_debug_mode(true)
        .with_library(library)
        .unwrap();

    let network_note = NoteBuilder::new(wallet.id(), client.rng())
        .code(
            "use.external_contract::counter_contract
            begin
                call.counter_contract::increment_count
            end",
        )
        .tag(NoteTag::from_account_id(network_account.id()).into())
        .build(&assembler)
        .unwrap();

    // Prepare both transactions
    let tx_request = TransactionRequestBuilder::new()
        .with_own_output_notes(vec![OutputNote::Full(network_note.clone())])
        .build()
        .unwrap();

    let bump_transaction = client.new_transaction(wallet.id(), tx_request).await.unwrap();
    client.testing_apply_transaction(bump_transaction.clone()).await.unwrap();

    let tx_request = TransactionRequestBuilder::new()
        .with_unauthenticated_input_notes(vec![(network_note, None)])
        .build()
        .unwrap();

    let consume_transaction =
        client.new_transaction(native_account.id(), tx_request).await.unwrap();

    let bump_proof = client.testing_prove_transaction(&bump_transaction).await.unwrap();
    let consume_proof = client.testing_prove_transaction(&consume_transaction).await.unwrap();

    // Submit both transactions
    client.testing_submit_proven_transaction(bump_proof).await.unwrap();
    client.testing_submit_proven_transaction(consume_proof).await.unwrap();

    client.testing_apply_transaction(consume_transaction).await.unwrap();

    wait_for_blocks(&mut client, 2).await;

    // The network account should have original value
    assert_eq!(
        client
            .get_account(network_account.id())
            .await
            .unwrap()
            .unwrap()
            .account()
            .storage()
            .get_item(0)
            .unwrap(),
        Digest::from([ZERO, ZERO, ZERO, Felt::new(1)])
    );

    // The native account should have the incremented value
    assert_eq!(
        client
            .get_account(native_account.id())
            .await
            .unwrap()
            .unwrap()
            .account()
            .storage()
            .get_item(0)
            .unwrap(),
        Digest::from([ZERO, ZERO, ZERO, Felt::new(2)])
    );
}
