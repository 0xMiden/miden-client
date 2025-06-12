use std::sync::Arc;

use miden_client::{
    Felt, Word, ZERO,
    account::{Account, AccountBuilder, AccountStorageMode, StorageSlot},
    note::{NoteExecutionMode, NoteTag},
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
use rand::RngCore;

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
async fn deploy_counter_contract(client: &mut TestClient) -> Result<(Account, Library), String> {
    let (acc, seed, library) = get_counter_contract_account(client).await;

    client.add_account(&acc, Some(seed), false).await.unwrap();

    let assembler: Assembler = TransactionKernel::assembler().with_debug_mode(true);
    let tx_script = TransactionScript::compile(
        "use.external_contract::counter_contract
        begin
            call.counter_contract::increment_count
        end",
        [],
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

async fn get_counter_contract_account(client: &mut TestClient) -> (Account, Word, Library) {
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
        .storage_mode(AccountStorageMode::Network)
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

    let (network_account, library) = deploy_counter_contract(&mut client).await.unwrap();

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
                .tag(
                    NoteTag::from_account_id(network_account.id(), NoteExecutionMode::Network)
                        .unwrap()
                        .into(),
                )
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
