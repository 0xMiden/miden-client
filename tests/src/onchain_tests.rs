use std::sync::Arc;

use miden_client::{
    Felt, Word, ZERO,
    account::{Account, AccountBuilder, StorageSlot, build_wallet_id},
    auth::AuthSecretKey,
    note::{NoteExecutionMode, NoteTag},
    store::{InputNoteState, NoteFilter},
    testing::{common::*, note::NoteBuilder},
    transaction::{
        OutputNote, PaymentTransactionData, TransactionRequestBuilder, TransactionScript,
    },
    utils::{
        execute_tx_and_sync, insert_new_fungible_faucet, insert_new_wallet,
        insert_new_wallet_with_seed, wait_for_blocks,
    },
};
use miden_lib::transaction::TransactionKernel;
use miden_objects::{
    Digest,
    account::{AccountComponent, AccountStorageMode},
    assembly::{Assembler, DefaultSourceManager, Library, LibraryPath, Module, ModuleKind},
    asset::{Asset, FungibleAsset},
    note::{NoteFile, NoteType, compute_note_commitment},
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
async fn test_onchain_notes_flow() {
    // Client 1 is an private faucet which will mint an onchain note for client 2
    let (mut client_1, keystore_1) = create_test_client().await;
    // Client 2 is an private account which will consume the note that it will sync from the node
    let (mut client_2, keystore_2) = create_test_client().await;
    // Client 3 will be transferred part of the assets by client 2's account
    let (mut client_3, keystore_3) = create_test_client().await;
    wait_for_node(&mut client_3).await;

    // Create faucet account
    let (faucet_account, ..) =
        insert_new_fungible_faucet(&mut client_1, AccountStorageMode::Private, &keystore_1)
            .await
            .unwrap();

    // Create regular accounts
    let (basic_wallet_1, ..) =
        insert_new_wallet(&mut client_2, AccountStorageMode::Private, &keystore_2)
            .await
            .unwrap();

    // Create regular accounts
    let (basic_wallet_2, ..) =
        insert_new_wallet(&mut client_3, AccountStorageMode::Private, &keystore_3)
            .await
            .unwrap();

    client_1.sync_state().await.unwrap();
    client_2.sync_state().await.unwrap();

    let tx_request = TransactionRequestBuilder::new()
        .build_mint_fungible_asset(
            FungibleAsset::new(faucet_account.id(), MINT_AMOUNT).unwrap(),
            basic_wallet_1.id(),
            NoteType::Public,
            client_1.rng(),
        )
        .unwrap();
    let note = tx_request.expected_output_notes().next().unwrap().clone();
    execute_tx_and_sync(&mut client_1, faucet_account.id(), tx_request)
        .await
        .unwrap();

    // Client 2's account should receive the note here:
    client_2.sync_state().await.unwrap();

    // Assert that the note is the same
    let received_note = client_2.get_input_note(note.id()).await.unwrap().unwrap();
    assert_eq!(
        compute_note_commitment(received_note.id(), received_note.metadata().unwrap()),
        note.commitment()
    );

    // consume the note
    consume_notes(&mut client_2, basic_wallet_1.id(), &[received_note]).await;
    assert_account_has_single_asset(
        &client_2,
        basic_wallet_1.id(),
        faucet_account.id(),
        MINT_AMOUNT,
    )
    .await;

    let p2id_asset = FungibleAsset::new(faucet_account.id(), TRANSFER_AMOUNT).unwrap();
    let tx_request = TransactionRequestBuilder::new()
        .build_pay_to_id(
            PaymentTransactionData::new(
                vec![p2id_asset.into()],
                basic_wallet_1.id(),
                basic_wallet_2.id(),
            ),
            None,
            NoteType::Public,
            client_2.rng(),
        )
        .unwrap();
    execute_tx_and_sync(&mut client_2, basic_wallet_1.id(), tx_request)
        .await
        .unwrap();

    // Create a note for client 3 that is already consumed before syncing
    let tx_request = TransactionRequestBuilder::new()
        .build_pay_to_id(
            PaymentTransactionData::new(
                vec![p2id_asset.into()],
                basic_wallet_1.id(),
                basic_wallet_2.id(),
            ),
            Some(1.into()),
            NoteType::Public,
            client_2.rng(),
        )
        .unwrap();
    let note = tx_request.expected_output_notes().next().unwrap().clone();
    execute_tx_and_sync(&mut client_2, basic_wallet_1.id(), tx_request)
        .await
        .unwrap();

    let tx_request = TransactionRequestBuilder::new().build_consume_notes(vec![note.id()]).unwrap();
    execute_tx_and_sync(&mut client_2, basic_wallet_1.id(), tx_request)
        .await
        .unwrap();

    // sync client 3 (basic account 2)
    client_3.sync_state().await.unwrap();

    // client 3 should have two notes, the one directed to them and the one consumed by client 2
    // (which should come from the tag added)
    assert_eq!(client_3.get_input_notes(NoteFilter::Committed).await.unwrap().len(), 1);
    assert_eq!(client_3.get_input_notes(NoteFilter::Consumed).await.unwrap().len(), 1);

    let note = client_3
        .get_input_notes(NoteFilter::Committed)
        .await
        .unwrap()
        .first()
        .unwrap()
        .clone();

    consume_notes(&mut client_3, basic_wallet_2.id(), &[note]).await;
    assert_account_has_single_asset(
        &client_3,
        basic_wallet_2.id(),
        faucet_account.id(),
        TRANSFER_AMOUNT,
    )
    .await;
}

#[tokio::test]
async fn test_onchain_accounts() {
    let (mut client_1, keystore_1) = create_test_client().await;
    let (mut client_2, keystore_2) = create_test_client().await;
    wait_for_node(&mut client_2).await;

    let (faucet_account_header, _, secret_key) =
        insert_new_fungible_faucet(&mut client_1, AccountStorageMode::Public, &keystore_1)
            .await
            .unwrap();

    let (first_regular_account, ..) =
        insert_new_wallet(&mut client_1, AccountStorageMode::Private, &keystore_1)
            .await
            .unwrap();

    let (second_client_first_regular_account, ..) =
        insert_new_wallet(&mut client_2, AccountStorageMode::Private, &keystore_2)
            .await
            .unwrap();

    let target_account_id = first_regular_account.id();
    let second_client_target_account_id = second_client_first_regular_account.id();
    let faucet_account_id = faucet_account_header.id();

    let (_, status) = client_1.get_account_header_by_id(faucet_account_id).await.unwrap().unwrap();
    let faucet_seed = status.seed().cloned();

    keystore_2.add_key(&AuthSecretKey::RpoFalcon512(secret_key)).unwrap();
    client_2.add_account(&faucet_account_header, faucet_seed, false).await.unwrap();

    // First Mint necesary token
    println!("First client consuming note");
    client_1.sync_state().await.unwrap();
    let note =
        mint_note(&mut client_1, target_account_id, faucet_account_id, NoteType::Private).await;

    // Update the state in the other client and ensure the onchain faucet commitment is consistent
    // between clients
    client_2.sync_state().await.unwrap();

    let (client_1_faucet, _) = client_1
        .get_account_header_by_id(faucet_account_header.id())
        .await
        .unwrap()
        .unwrap();
    let (client_2_faucet, _) = client_2
        .get_account_header_by_id(faucet_account_header.id())
        .await
        .unwrap()
        .unwrap();

    assert_eq!(client_1_faucet.commitment(), client_2_faucet.commitment());

    // Now use the faucet in the second client to mint to its own account
    println!("Second client consuming note");
    let second_client_note = mint_note(
        &mut client_2,
        second_client_target_account_id,
        faucet_account_id,
        NoteType::Private,
    )
    .await;

    // Update the state in the other client and ensure the onchain faucet commitment is consistent
    // between clients
    client_1.sync_state().await.unwrap();

    println!("About to consume");
    consume_notes(&mut client_1, target_account_id, &[note]).await;
    assert_account_has_single_asset(&client_1, target_account_id, faucet_account_id, MINT_AMOUNT)
        .await;
    consume_notes(&mut client_2, second_client_target_account_id, &[second_client_note]).await;
    assert_account_has_single_asset(
        &client_2,
        second_client_target_account_id,
        faucet_account_id,
        MINT_AMOUNT,
    )
    .await;

    let (client_1_faucet, _) = client_1
        .get_account_header_by_id(faucet_account_header.id())
        .await
        .unwrap()
        .unwrap();
    let (client_2_faucet, _) = client_2
        .get_account_header_by_id(faucet_account_header.id())
        .await
        .unwrap()
        .unwrap();

    assert_eq!(client_1_faucet.commitment(), client_2_faucet.commitment());

    // Now we'll try to do a p2id transfer from an account of one client to the other one
    let from_account_id = target_account_id;
    let to_account_id = second_client_target_account_id;

    // get initial balances
    let from_account_balance = client_1
        .get_account(from_account_id)
        .await
        .unwrap()
        .unwrap()
        .account()
        .vault()
        .get_balance(faucet_account_id)
        .unwrap_or(0);
    let to_account_balance = client_2
        .get_account(to_account_id)
        .await
        .unwrap()
        .unwrap()
        .account()
        .vault()
        .get_balance(faucet_account_id)
        .unwrap_or(0);

    let asset = FungibleAsset::new(faucet_account_id, TRANSFER_AMOUNT).unwrap();

    println!("Running P2ID tx...");
    let tx_request = TransactionRequestBuilder::new()
        .build_pay_to_id(
            PaymentTransactionData::new(
                vec![Asset::Fungible(asset)],
                from_account_id,
                to_account_id,
            ),
            None,
            NoteType::Public,
            client_1.rng(),
        )
        .unwrap();
    execute_tx_and_sync(&mut client_1, from_account_id, tx_request).await.unwrap();

    // sync on second client until we receive the note
    println!("Syncing on second client...");
    client_2.sync_state().await.unwrap();
    let notes = client_2.get_input_notes(NoteFilter::Committed).await.unwrap();

    //Import the note on the first client so that we can later check its consumer account
    client_1.import_note(NoteFile::NoteId(notes[0].id())).await.unwrap();

    // Consume the note
    println!("Consuming note on second client...");
    let tx_request = TransactionRequestBuilder::new()
        .build_consume_notes(vec![notes[0].id()])
        .unwrap();
    execute_tx_and_sync(&mut client_2, to_account_id, tx_request).await.unwrap();

    // sync on first client
    println!("Syncing on first client...");
    client_1.sync_state().await.unwrap();

    // Check that the client doesn't know who consumed the note
    let input_note = client_1.get_input_note(notes[0].id()).await.unwrap().unwrap();
    assert!(matches!(input_note.state(), InputNoteState::ConsumedExternal { .. }));

    let new_from_account_balance = client_1
        .get_account(from_account_id)
        .await
        .unwrap()
        .unwrap()
        .account()
        .vault()
        .get_balance(faucet_account_id)
        .unwrap_or(0);
    let new_to_account_balance = client_2
        .get_account(to_account_id)
        .await
        .unwrap()
        .unwrap()
        .account()
        .vault()
        .get_balance(faucet_account_id)
        .unwrap_or(0);

    assert_eq!(new_from_account_balance, from_account_balance - TRANSFER_AMOUNT);
    assert_eq!(new_to_account_balance, to_account_balance + TRANSFER_AMOUNT);
}

#[tokio::test]
async fn test_import_account_by_id() {
    let (mut client_1, keystore_1) = create_test_client().await;
    let (mut client_2, keystore_2) = create_test_client().await;
    wait_for_node(&mut client_1).await;

    let mut user_seed = [0u8; 32];
    client_1.rng().fill_bytes(&mut user_seed);

    let (faucet_account_header, ..) =
        insert_new_fungible_faucet(&mut client_1, AccountStorageMode::Public, &keystore_1)
            .await
            .unwrap();

    let (first_regular_account, _, secret_key) = insert_new_wallet_with_seed(
        &mut client_1,
        AccountStorageMode::Public,
        &keystore_1,
        user_seed,
    )
    .await
    .unwrap();

    let target_account_id = first_regular_account.id();
    let faucet_account_id = faucet_account_header.id();

    // First mint and consume in the first client
    mint_and_consume(&mut client_1, target_account_id, faucet_account_id, NoteType::Public).await;

    // Mint a note for the second client
    let note =
        mint_note(&mut client_1, target_account_id, faucet_account_id, NoteType::Public).await;

    // Import the public account by id
    let built_wallet_id =
        build_wallet_id(user_seed, secret_key.public_key(), AccountStorageMode::Public, false)
            .unwrap();
    assert_eq!(built_wallet_id, first_regular_account.id());
    client_2.import_account_by_id(built_wallet_id).await.unwrap();
    keystore_2.add_key(&AuthSecretKey::RpoFalcon512(secret_key)).unwrap();

    let original_account = client_1.get_account(first_regular_account.id()).await.unwrap().unwrap();
    let imported_account = client_2.get_account(first_regular_account.id()).await.unwrap().unwrap();
    assert_eq!(imported_account.account().commitment(), original_account.account().commitment());

    // Now use the wallet in the second client to consume the generated note
    println!("Second client consuming note");
    client_2.sync_state().await.unwrap();
    consume_notes(&mut client_2, target_account_id, &[note]).await;
    assert_account_has_single_asset(
        &client_2,
        target_account_id,
        faucet_account_id,
        MINT_AMOUNT * 2,
    )
    .await;
}

#[tokio::test]
async fn test_counter_contract_ntx() {
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

    execute_tx_and_sync(&mut client, native_account.id(), tx_request).await.unwrap();

    wait_for_blocks(&mut client, 2).await.unwrap();

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
