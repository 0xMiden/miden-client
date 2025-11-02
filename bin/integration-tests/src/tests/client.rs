use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use miden_client::account::{AccountId, AccountStorageMode, StorageMap, StorageSlot};
use miden_client::assembly::{DefaultSourceManager, LibraryPath, Module, ModuleKind};
use miden_client::asset::{Asset, FungibleAsset};
use miden_client::builder::ClientBuilder;
use miden_client::keystore::FilesystemKeyStore;
use miden_client::note::{Note, NoteAssets, NoteFile, NoteMetadata, NoteScript, NoteTag, NoteType};
use miden_client::rpc::domain::account::FetchedAccount;
use miden_client::store::{
    InputNoteRecord,
    InputNoteState,
    NoteFilter,
    OutputNoteState,
    TransactionFilter,
};
use miden_client::testing::account_id::ACCOUNT_ID_NATIVE_ASSET_FAUCET;
use miden_client::testing::common::*;
use miden_client::transaction::{
    DiscardCause,
    PaymentNoteDescription,
    ProvenTransaction,
    TransactionInputs,
    TransactionKernel,
    TransactionProver,
    TransactionProverError,
    TransactionRequestBuilder,
    TransactionStatus,
};
use miden_client::{BlockNumber, ClientError, Felt, ScriptBuilder};
use miden_client_sqlite_store::ClientBuilderSqliteExt;

use crate::tests::config::ClientConfig;

pub async fn test_client_builder_initializes_client_with_endpoint(
    client_config: ClientConfig,
) -> Result<()> {
    let (endpoint, _, store_config, auth_path) = client_config.as_parts();

    let mut client = ClientBuilder::<FilesystemKeyStore<_>>::new()
        .grpc_client(&endpoint, Some(10_000))
        .filesystem_keystore(auth_path.to_str().context("failed to convert auth path to string")?)
        .sqlite_store(store_config)
        .in_debug_mode(miden_client::DebugMode::Enabled)
        .build()
        .await?;

    assert!(client.in_debug_mode());

    let sync_summary = client.sync_state().await?;

    assert!(sync_summary.block_num.as_u32() > 0);
    Ok(())
}

pub async fn test_multiple_tx_on_same_block(client_config: ClientConfig) -> Result<()> {
    let (mut client, authenticator) = client_config.into_client().await?;
    wait_for_node(&mut client).await;

    let (first_regular_account, second_regular_account, faucet_account_header) =
        setup_two_wallets_and_faucet(&mut client, AccountStorageMode::Private, &authenticator)
            .await?;

    let from_account_id = first_regular_account.id();
    let to_account_id = second_regular_account.id();
    let faucet_account_id = faucet_account_header.id();

    // First Mint necessary token
    let tx_id =
        mint_and_consume(&mut client, from_account_id, faucet_account_id, NoteType::Private).await;
    wait_for_tx(&mut client, tx_id).await?;

    // Do a transfer from first account to second account
    let asset = FungibleAsset::new(faucet_account_id, TRANSFER_AMOUNT).unwrap();
    let tx_request_1 = TransactionRequestBuilder::new()
        .build_pay_to_id(
            PaymentNoteDescription::new(
                vec![Asset::Fungible(asset)],
                from_account_id,
                to_account_id,
            ),
            NoteType::Private,
            client.rng(),
        )
        .unwrap();
    let tx_request_2 = TransactionRequestBuilder::new()
        .build_pay_to_id(
            PaymentNoteDescription::new(
                vec![Asset::Fungible(asset)],
                from_account_id,
                to_account_id,
            ),
            NoteType::Private,
            client.rng(),
        )
        .unwrap();

    println!("Running P2ID tx...");

    // Create transactions
    let transaction_execution_result_1 =
        client.new_transaction(from_account_id, tx_request_1).await.unwrap();
    let transaction_id_1 = transaction_execution_result_1.executed_transaction().id();
    let tx_prove_1 =
        client.testing_prove_transaction(&transaction_execution_result_1).await.unwrap();
    client
        .testing_apply_transaction(transaction_execution_result_1.clone())
        .await
        .unwrap();

    let transaction_execution_result_2 =
        client.new_transaction(from_account_id, tx_request_2).await.unwrap();
    let transaction_id_2 = transaction_execution_result_2.executed_transaction().id();
    let tx_prove_2 =
        client.testing_prove_transaction(&transaction_execution_result_2).await.unwrap();
    client
        .testing_apply_transaction(transaction_execution_result_2.clone())
        .await
        .unwrap();

    client.sync_state().await.unwrap();

    // wait for 1 block
    wait_for_blocks(&mut client, 1).await;

    // Submit the proven transactions
    client
        .testing_submit_proven_transaction(
            tx_prove_1,
            transaction_execution_result_1.executed_transaction().tx_inputs().clone(),
        )
        .await
        .unwrap();
    client
        .testing_submit_proven_transaction(
            tx_prove_2,
            transaction_execution_result_2.executed_transaction().tx_inputs().clone(),
        )
        .await
        .unwrap();

    // wait for 1 block
    wait_for_tx(&mut client, transaction_id_1).await?;

    let transactions = client
        .get_transactions(TransactionFilter::All)
        .await
        .unwrap()
        .into_iter()
        .filter(|tx| tx.id == transaction_id_1 || tx.id == transaction_id_2)
        .collect::<Vec<_>>();

    assert_eq!(transactions.len(), 2);
    assert!(matches!(transactions[0].status, TransactionStatus::Committed { .. }));
    assert_eq!(transactions[0].status, transactions[1].status);

    let note_id = transactions[0].details.output_notes.iter().next().unwrap().id();
    let note = client.get_output_note(note_id).await.unwrap().unwrap();
    assert!(matches!(note.state(), OutputNoteState::CommittedFull { .. }));

    let sender_account = client
        .get_account(from_account_id)
        .await?
        .context("failed to find sender account after  transactions")?;
    assert_eq!(
        sender_account.account().vault().get_balance(faucet_account_id).unwrap(),
        MINT_AMOUNT - (TRANSFER_AMOUNT * 2)
    );
    Ok(())
}

pub async fn test_import_expected_notes(client_config: ClientConfig) -> Result<()> {
    let (mut client_1, authenticator_1) = client_config.clone().into_client().await?;
    let (first_basic_account, faucet_account) =
        setup_wallet_and_faucet(&mut client_1, AccountStorageMode::Private, &authenticator_1)
            .await?;

    let (mut client_2, authenticator_2) = client_config.into_client().await?;
    let (client_2_account, _) =
        insert_new_wallet(&mut client_2, AccountStorageMode::Private, &authenticator_2).await?;

    wait_for_node(&mut client_2).await;

    let tx_request = TransactionRequestBuilder::new()
        .build_mint_fungible_asset(
            FungibleAsset::new(faucet_account.id(), MINT_AMOUNT).unwrap(),
            client_2_account.id(),
            NoteType::Public,
            client_2.rng(),
        )
        .unwrap();
    let note: InputNoteRecord =
        tx_request.expected_output_own_notes().pop().unwrap().clone().into();
    client_2.sync_state().await.unwrap();

    // Importing a public note before it's committed onchain should fail
    assert!(matches!(
        client_2.import_note(NoteFile::NoteId(note.id())).await.unwrap_err(),
        ClientError::NoteNotFoundOnChain(_)
    ));
    execute_tx_and_sync(&mut client_1, faucet_account.id(), tx_request).await?;

    // Use client 1 to wait until a couple of blocks have passed
    wait_for_blocks(&mut client_1, 3).await;

    let new_sync_data = client_2.sync_state().await.unwrap();

    client_2.add_note_tag(note.metadata().unwrap().tag()).await.unwrap();
    client_2.import_note(NoteFile::NoteId(note.clone().id())).await.unwrap();
    client_2.sync_state().await.unwrap();
    let input_note = client_2.get_input_note(note.id()).await.unwrap().unwrap();
    // If imported after execution and syncing then the inclusion proof should be Some
    assert!(input_note.inclusion_proof().is_some(), "Expected inclusion proof to be present");

    assert!(
        new_sync_data.block_num > input_note.inclusion_proof().unwrap().location().block_num() + 1
    );

    // If client 2 successfully consumes the note, we confirm we have MMR and block header data
    let tx_id =
        consume_notes(&mut client_2, client_2_account.id(), &[input_note.try_into().unwrap()])
            .await;
    wait_for_tx(&mut client_2, tx_id).await?;

    let tx_request = TransactionRequestBuilder::new()
        .build_mint_fungible_asset(
            FungibleAsset::new(faucet_account.id(), MINT_AMOUNT).unwrap(),
            first_basic_account.id(),
            NoteType::Private,
            client_2.rng(),
        )
        .unwrap();
    let note: InputNoteRecord =
        tx_request.expected_output_own_notes().pop().unwrap().clone().into();

    // Import the node before it's committed onchain works if we have full `NoteDetails`
    client_2.add_note_tag(note.metadata().unwrap().tag()).await.unwrap();
    client_2
        .import_note(NoteFile::NoteDetails {
            details: note.clone().into(),
            after_block_num: client_1.get_sync_height().await.unwrap(),
            tag: Some(note.metadata().unwrap().tag()),
        })
        .await
        .unwrap();
    let input_note = client_2.get_input_note(note.id()).await.unwrap().unwrap();

    // If imported before execution, the note should be imported in `Expected` state
    assert!(matches!(input_note.state(), InputNoteState::Expected { .. }));

    execute_tx_and_sync(&mut client_1, faucet_account.id(), tx_request).await?;
    client_2.sync_state().await.unwrap();

    // After sync, the imported note should have inclusion proof even if it's not relevant for its
    // accounts.
    let input_note = client_2.get_input_note(note.id()).await.unwrap().unwrap();
    assert!(input_note.inclusion_proof().is_some(), "Expected inclusion proof to be present");

    // If inclusion proof is invalid this should panic
    let tx_id =
        consume_notes(&mut client_1, first_basic_account.id(), &[input_note.try_into().unwrap()])
            .await;
    wait_for_tx(&mut client_1, tx_id).await?;
    Ok(())
}

pub async fn test_import_expected_note_uncommitted(client_config: ClientConfig) -> Result<()> {
    let (mut client_1, authenticator) = client_config.clone().into_client().await?;
    let faucet_account =
        insert_new_fungible_faucet(&mut client_1, AccountStorageMode::Private, &authenticator)
            .await
            .unwrap()
            .0;

    let (mut client_2, _) = ClientConfig::default()
        .with_rpc_endpoint(client_config.rpc_endpoint())
        .into_client()
        .await?;
    let (client_2_account, _) =
        insert_new_wallet(&mut client_2, AccountStorageMode::Private, &authenticator).await?;

    wait_for_node(&mut client_2).await;

    let tx_request = TransactionRequestBuilder::new().build_mint_fungible_asset(
        FungibleAsset::new(faucet_account.id(), MINT_AMOUNT).unwrap(),
        client_2_account.id(),
        NoteType::Public,
        client_1.rng(),
    )?;

    let note: InputNoteRecord =
        tx_request.expected_output_own_notes().pop().unwrap().clone().into();
    client_2.sync_state().await.unwrap();

    // If the verification is requested before execution then the import should fail
    let imported_note_id = client_2
        .import_note(NoteFile::NoteDetails {
            details: note.into(),
            after_block_num: 0.into(),
            tag: None,
        })
        .await?;

    let imported_note = client_2.get_input_note(imported_note_id).await.unwrap().unwrap();

    assert!(matches!(imported_note.state(), InputNoteState::Expected { .. }));
    Ok(())
}

pub async fn test_import_expected_notes_from_the_past_as_committed(
    client_config: ClientConfig,
) -> Result<()> {
    let (mut client_1, authenticator_1) = client_config.clone().into_client().await?;
    let (first_basic_account, faucet_account) =
        setup_wallet_and_faucet(&mut client_1, AccountStorageMode::Private, &authenticator_1)
            .await?;

    let (mut client_2, _) = ClientConfig::default()
        .with_rpc_endpoint(client_config.rpc_endpoint())
        .into_client()
        .await?;

    wait_for_node(&mut client_2).await;

    let tx_request = TransactionRequestBuilder::new().build_mint_fungible_asset(
        FungibleAsset::new(faucet_account.id(), MINT_AMOUNT).unwrap(),
        first_basic_account.id(),
        NoteType::Public,
        client_1.rng(),
    )?;
    let note: InputNoteRecord =
        tx_request.expected_output_own_notes().pop().unwrap().clone().into();

    let block_height_before = client_1.get_sync_height().await.unwrap();

    execute_tx_and_sync(&mut client_1, faucet_account.id(), tx_request).await?;

    // Use client 1 to wait until a couple of blocks have passed
    wait_for_blocks(&mut client_1, 3).await;

    // importing the note before client_2 is synced will result in a note with `Expected` state
    let note_id = client_2
        .import_note(NoteFile::NoteDetails {
            details: note.clone().into(),
            after_block_num: block_height_before,
            tag: Some(note.metadata().unwrap().tag()),
        })
        .await?;

    let imported_note = client_2.get_input_note(note_id).await.unwrap().unwrap();

    assert!(matches!(imported_note.state(), InputNoteState::Expected { .. }));

    client_2.sync_state().await.unwrap();

    // import the note after syncing the client
    let note_id = client_2
        .import_note(NoteFile::NoteDetails {
            details: note.clone().into(),
            after_block_num: block_height_before,
            tag: Some(note.metadata().unwrap().tag()),
        })
        .await?;

    let imported_note = client_2.get_input_note(note_id).await.unwrap().unwrap();

    // Get the note status in client 1
    let client_1_note = client_1.get_input_note(note_id).await.unwrap().unwrap();

    assert_eq!(imported_note.state(), client_1_note.state());
    assert!(matches!(imported_note.state(), InputNoteState::Committed { .. }));
    Ok(())
}

pub async fn test_get_account_update(client_config: ClientConfig) -> Result<()> {
    // Create a client with both public and private accounts.
    let (mut client, authenticator) = client_config.clone().into_client().await?;

    let (basic_wallet_1, faucet_account) =
        setup_wallet_and_faucet(&mut client, AccountStorageMode::Private, &authenticator).await?;
    wait_for_node(&mut client).await;

    let (basic_wallet_2, ..) =
        insert_new_wallet(&mut client, AccountStorageMode::Public, &authenticator).await?;

    // Mint and consume notes with both accounts so they are included in the node.
    let tx_id_1 =
        mint_and_consume(&mut client, basic_wallet_1.id(), faucet_account.id(), NoteType::Private)
            .await;
    wait_for_tx(&mut client, tx_id_1).await?;
    let tx_id_2 =
        mint_and_consume(&mut client, basic_wallet_2.id(), faucet_account.id(), NoteType::Private)
            .await;
    wait_for_tx(&mut client, tx_id_2).await?;

    // Request updates from node for both accounts. The request should not fail and both types of
    // [`AccountDetails`] should be received.
    // TODO: should we expose the `get_account_update` endpoint from the Client?
    let rpc_api = client.test_rpc_api();
    let details1 = rpc_api.get_account_details(basic_wallet_1.id()).await.unwrap();
    let details2 = rpc_api.get_account_details(basic_wallet_2.id()).await.unwrap();

    assert!(matches!(details1, FetchedAccount::Private(_, _)));
    assert!(matches!(details2, FetchedAccount::Public(_, _)));
    Ok(())
}

pub async fn test_sync_detail_values(client_config: ClientConfig) -> Result<()> {
    let (mut client1, authenticator_1) = client_config.clone().into_client().await?;
    let (mut client2, authenticator_2) = ClientConfig::default()
        .with_rpc_endpoint(client_config.rpc_endpoint())
        .into_client()
        .await?;
    wait_for_node(&mut client1).await;
    wait_for_node(&mut client2).await;

    let (first_regular_account, faucet_account_header) =
        setup_wallet_and_faucet(&mut client1, AccountStorageMode::Private, &authenticator_1)
            .await?;

    let (second_regular_account, ..) =
        insert_new_wallet(&mut client2, AccountStorageMode::Private, &authenticator_2).await?;

    let from_account_id = first_regular_account.id();
    let to_account_id = second_regular_account.id();
    let faucet_account_id = faucet_account_header.id();

    // First Mint necessary token
    let tx_id =
        mint_and_consume(&mut client1, from_account_id, faucet_account_id, NoteType::Private).await;
    wait_for_tx(&mut client1, tx_id).await?;

    // Second client sync shouldn't have any new changes
    let new_details = client2.sync_state().await.unwrap();
    assert!(new_details.is_empty());

    // Do a transfer with recall from first account to second account
    let asset = FungibleAsset::new(faucet_account_id, TRANSFER_AMOUNT).unwrap();
    let tx_request = TransactionRequestBuilder::new().build_pay_to_id(
        PaymentNoteDescription::new(vec![Asset::Fungible(asset)], from_account_id, to_account_id)
            .with_reclaim_height(new_details.block_num + 5),
        NoteType::Public,
        client1.rng(),
    )?;
    let note_id = tx_request.expected_output_own_notes().pop().unwrap().id();
    execute_tx_and_sync(&mut client1, from_account_id, tx_request).await?;

    // Second client sync should have new note
    let new_details = client2.sync_state().await.unwrap();
    assert_eq!(new_details.new_public_notes.len(), 1);
    assert_eq!(new_details.committed_notes.len(), 0);
    assert_eq!(new_details.consumed_notes.len(), 0);
    assert_eq!(new_details.updated_accounts.len(), 0);

    // Consume the note with the second account
    let tx_request = TransactionRequestBuilder::new().build_consume_notes(vec![note_id]).unwrap();
    execute_tx_and_sync(&mut client2, to_account_id, tx_request).await?;

    // First client sync should have a new nullifier as the note was consumed
    let new_details = client1.sync_state().await.unwrap();
    assert_eq!(new_details.committed_notes.len(), 0);
    assert_eq!(new_details.consumed_notes.len(), 1);
    Ok(())
}

/// This test runs 3 mint transactions that get included in different blocks so that once we sync
/// we can check that each transaction gets marked as committed in the corresponding block.
pub async fn test_multiple_transactions_can_be_committed_in_different_blocks_without_sync(
    client_config: ClientConfig,
) -> Result<()> {
    let (mut client, authenticator) = client_config.into_client().await?;

    let (first_regular_account, faucet_account_header) =
        setup_wallet_and_faucet(&mut client, AccountStorageMode::Private, &authenticator).await?;

    let from_account_id = first_regular_account.id();
    let faucet_account_id = faucet_account_header.id();

    // Mint first note
    let (first_note_id, first_note_tx_id) = {
        // Create a Mint Tx for 1000 units of our fungible asset
        let fungible_asset = FungibleAsset::new(faucet_account_id, MINT_AMOUNT).unwrap();

        println!("Minting Asset");
        let tx_request = TransactionRequestBuilder::new().build_mint_fungible_asset(
            fungible_asset,
            from_account_id,
            NoteType::Private,
            client.rng(),
        )?;

        println!("Executing transaction...");
        let transaction_execution_result =
            client.new_transaction(faucet_account_id, tx_request.clone()).await.unwrap();
        let transaction_id = transaction_execution_result.executed_transaction().id();

        println!("Sending transaction to node");
        let note_id = tx_request.expected_output_own_notes().pop().unwrap().id();
        client.submit_transaction(transaction_execution_result).await.unwrap();

        (note_id, transaction_id)
    };

    // Mint second note
    let (second_note_id, second_note_tx_id) = {
        // Create a Mint Tx for 1000 units of our fungible asset
        let fungible_asset = FungibleAsset::new(faucet_account_id, MINT_AMOUNT).unwrap();

        println!("Minting Asset");
        let tx_request = TransactionRequestBuilder::new().build_mint_fungible_asset(
            fungible_asset,
            from_account_id,
            NoteType::Private,
            client.rng(),
        )?;

        println!("Executing transaction...");
        let transaction_execution_result =
            client.new_transaction(faucet_account_id, tx_request.clone()).await.unwrap();
        let transaction_id = transaction_execution_result.executed_transaction().id();

        println!("Sending transaction to node");
        // May need a few attempts until it gets included
        let note_id = tx_request.expected_output_own_notes().pop().unwrap().id();
        while client
            .test_rpc_api()
            .get_notes_by_id(&[first_note_id])
            .await
            .unwrap()
            .is_empty()
        {
            std::thread::sleep(Duration::from_secs(3));
        }
        client.submit_transaction(transaction_execution_result).await.unwrap();

        (note_id, transaction_id)
    };

    // Mint third note
    let (third_note_id, third_note_tx_id) = {
        // Create a Mint Tx for 1000 units of our fungible asset
        let fungible_asset = FungibleAsset::new(faucet_account_id, MINT_AMOUNT).unwrap();

        println!("Minting Asset");
        let tx_request = TransactionRequestBuilder::new().build_mint_fungible_asset(
            fungible_asset,
            from_account_id,
            NoteType::Private,
            client.rng(),
        )?;

        println!("Executing transaction...");
        let transaction_execution_result =
            client.new_transaction(faucet_account_id, tx_request.clone()).await.unwrap();
        let transaction_id = transaction_execution_result.executed_transaction().id();

        println!("Sending transaction to node");
        // May need a few attempts until it gets included
        let note_id = tx_request.expected_output_own_notes().pop().unwrap().id();
        while client
            .test_rpc_api()
            .get_notes_by_id(&[second_note_id])
            .await
            .unwrap()
            .is_empty()
        {
            std::thread::sleep(Duration::from_secs(3));
        }
        client.submit_transaction(transaction_execution_result).await.unwrap();

        (note_id, transaction_id)
    };

    // Wait until the note gets committed in the node (without syncing)
    while client
        .test_rpc_api()
        .get_notes_by_id(&[third_note_id])
        .await
        .unwrap()
        .is_empty()
    {
        std::thread::sleep(Duration::from_secs(3));
    }

    client.sync_state().await.unwrap();

    let all_transactions = client.get_transactions(TransactionFilter::All).await.unwrap();
    let first_tx = all_transactions.iter().find(|tx| tx.id == first_note_tx_id).unwrap();
    let second_tx = all_transactions.iter().find(|tx| tx.id == second_note_tx_id).unwrap();
    let third_tx = all_transactions.iter().find(|tx| tx.id == third_note_tx_id).unwrap();

    match (first_tx.status.clone(), second_tx.status.clone(), third_tx.status.clone()) {
        (
            TransactionStatus::Committed { block_number: first_tx_commit_height, .. },
            TransactionStatus::Committed {
                block_number: second_tx_commit_height, ..
            },
            TransactionStatus::Committed { block_number: third_tx_commit_height, .. },
        ) => {
            assert!(first_tx_commit_height < second_tx_commit_height);
            assert!(second_tx_commit_height < third_tx_commit_height);
        },
        _ => {
            panic!("All three TXs should be committed in different blocks")
        },
    }
    Ok(())
}

/// Test that checks multiple features:
/// - Consuming multiple notes in a single transaction.
/// - Consuming authenticated notes.
/// - Consuming unauthenticated notes.
pub async fn test_consume_multiple_expected_notes(client_config: ClientConfig) -> Result<()> {
    let (mut client, authenticator_1) = client_config.clone().into_client().await?;
    let (mut unauth_client, authenticator_2) = ClientConfig::default()
        .with_rpc_endpoint(client_config.rpc_endpoint())
        .into_client()
        .await?;

    wait_for_node(&mut client).await;

    // Setup accounts
    let (target_basic_account_1, faucet_account_header) =
        setup_wallet_and_faucet(&mut client, AccountStorageMode::Private, &authenticator_1).await?;
    let (target_basic_account_2, ..) =
        insert_new_wallet(&mut unauth_client, AccountStorageMode::Private, &authenticator_2)
            .await?;
    unauth_client.sync_state().await.unwrap();

    let faucet_account_id = faucet_account_header.id();
    let to_account_ids = [target_basic_account_1.id(), target_basic_account_2.id()];

    // Mint tokens to the accounts
    let fungible_asset = FungibleAsset::new(faucet_account_id, TRANSFER_AMOUNT).unwrap();
    let mint_tx_request = mint_multiple_fungible_asset(
        fungible_asset,
        &[to_account_ids[0], to_account_ids[0], to_account_ids[1], to_account_ids[1]],
        NoteType::Private,
        client.rng(),
    );

    execute_tx_and_sync(&mut client, faucet_account_id, mint_tx_request.clone()).await?;
    unauth_client.sync_state().await.unwrap();

    // Filter notes by ownership
    let expected_notes = mint_tx_request.expected_output_own_notes().into_iter();
    let client_notes: Vec<_> = client.get_input_notes(NoteFilter::All).await.unwrap();
    let client_notes_ids: Vec<_> = client_notes.iter().map(|note| note.id()).collect();

    let (client_owned_notes, unauth_owned_notes): (Vec<_>, Vec<_>) =
        expected_notes.partition(|note| client_notes_ids.contains(&note.id()));

    // Create and execute transactions
    let tx_request_1 = TransactionRequestBuilder::new()
        .authenticated_input_notes(client_owned_notes.iter().map(|note| (note.id(), None)))
        .build()?;

    let tx_request_2 = TransactionRequestBuilder::new()
        .unauthenticated_input_notes(unauth_owned_notes.iter().map(|note| ((*note).clone(), None)))
        .build()?;

    let tx_id_1 = submit_and_await_tx(&mut client, to_account_ids[0], tx_request_1).await;
    let tx_id_2 = submit_and_await_tx(&mut unauth_client, to_account_ids[1], tx_request_2).await;

    // Ensure notes are processed
    assert!(!client.get_input_notes(NoteFilter::Processing).await.unwrap().is_empty());
    assert!(!unauth_client.get_input_notes(NoteFilter::Processing).await.unwrap().is_empty());

    wait_for_tx(&mut client, tx_id_1).await?;
    wait_for_tx(&mut unauth_client, tx_id_2).await?;

    // Verify no remaining expected notes and all notes are consumed
    assert!(client.get_input_notes(NoteFilter::Expected).await.unwrap().is_empty());
    assert!(unauth_client.get_input_notes(NoteFilter::Expected).await.unwrap().is_empty());

    assert!(
        !client.get_input_notes(NoteFilter::Consumed).await.unwrap().is_empty(),
        "Authenticated notes are consumed"
    );
    assert!(
        !unauth_client.get_input_notes(NoteFilter::Consumed).await.unwrap().is_empty(),
        "Unauthenticated notes are consumed"
    );

    // Validate the final asset amounts in each account
    for (client, account_id) in
        vec![(client, to_account_ids[0]), (unauth_client, to_account_ids[1])]
    {
        assert_account_has_single_asset(
            &client,
            account_id,
            faucet_account_id,
            TRANSFER_AMOUNT * 2,
        )
        .await;
    }
    Ok(())
}

pub async fn test_exploit(client_config: ClientConfig) -> Result<()> {
    let (mut client, authenticator_1) = client_config.clone().into_client().await?;
    let (mut unauth_client, authenticator_2) = ClientConfig::default()
        .with_rpc_endpoint(client_config.rpc_endpoint())
        .into_client()
        .await?;

    wait_for_node(&mut client).await;

    // Setup accounts
    let (target_basic_account_1, faucet_account_header) =
        setup_wallet_and_faucet(&mut client, AccountStorageMode::Private, &authenticator_1).await?;
    let (target_basic_account_2, ..) =
        insert_new_wallet(&mut unauth_client, AccountStorageMode::Private, &authenticator_2)
            .await?;

    unauth_client.sync_state().await.unwrap();

    let faucet_account_id = faucet_account_header.id();
    let to_account_ids = [target_basic_account_1.id(), target_basic_account_2.id()];

    // Mint tokens to the accounts
    let fungible_asset = FungibleAsset::new(faucet_account_id, TRANSFER_AMOUNT).unwrap();
    let mint_tx_request = mint_multiple_fungible_asset(
        fungible_asset,
        &[to_account_ids[0],to_account_ids[0],to_account_ids[1],to_account_ids[1]],
        NoteType::Private,
        client.rng(),
    );

    execute_tx_and_sync(&mut client, faucet_account_id, mint_tx_request.clone()).await?;
    unauth_client.sync_state().await.unwrap();

    // Filter notes by ownership
    let expected_notes = mint_tx_request.expected_output_own_notes().into_iter();

    let client_notes: Vec<_> = client.get_input_notes(NoteFilter::All).await.unwrap();
    let client_notes_ids: Vec<_> = client_notes.iter().map(|note| note.id()).collect();

    let (client_owned_notes, unauth_owned_notes): (Vec<_>, Vec<_>) =
        expected_notes.partition(|note| client_notes_ids.contains(&note.id()));

    // Create and execute transactions
    let tx_request_1 = TransactionRequestBuilder::new()
        .authenticated_input_notes(client_owned_notes.iter().map(|note| (note.id(), None)))
        .build()?;

    // modify the note

    let note = unauth_owned_notes[0].clone();
    let metadata = NoteMetadata::new(
        ACCOUNT_ID_NATIVE_ASSET_FAUCET.try_into()?,
        NoteType::Public,
        NoteTag::from_account_id(ACCOUNT_ID_NATIVE_ASSET_FAUCET.try_into()?),
        miden_client::note::NoteExecutionHint::Always,
        Felt::new(0),
    ).unwrap();
    let modified_note = Note::new(note.assets().clone(), metadata, note.recipient().clone());

    let tx_request_2 = TransactionRequestBuilder::new()
        .unauthenticated_input_notes([(modified_note,None)])
        .build()?;

    let tx_id_1 = submit_and_await_tx(&mut client, to_account_ids[0], tx_request_1).await;
    wait_for_tx(&mut client, tx_id_1).await?;

    let tx_id_2 = submit_and_await_tx(&mut unauth_client, to_account_ids[1], tx_request_2).await;

    println!("both transactions went through");
    // Ensure notes are processed
    //assert!(!client.get_input_notes(NoteFilter::Processing).await.unwrap().is_empty());
    assert!(!unauth_client.get_input_notes(NoteFilter::Processing).await.unwrap().is_empty());

    wait_for_tx(&mut client, tx_id_1).await?;
    std::println!("asdfsfd");
    wait_for_tx(&mut unauth_client, tx_id_2).await?;

    // Verify no remaining expected notes and all notes are consumed
    assert!(client.get_input_notes(NoteFilter::Expected).await.unwrap().is_empty());
    assert!(unauth_client.get_input_notes(NoteFilter::Expected).await.unwrap().is_empty());

    assert!(
        !client.get_input_notes(NoteFilter::Consumed).await.unwrap().is_empty(),
        "Authenticated notes are consumed"
    );
    assert!(
        !unauth_client.get_input_notes(NoteFilter::Consumed).await.unwrap().is_empty(),
        "Unauthenticated notes are consumed"
    );

    // Validate the final asset amounts in each account
    for (client, account_id) in
        vec![(client, to_account_ids[0]), (unauth_client, to_account_ids[1])]
    {
        assert_account_has_single_asset(
            &client,
            account_id,
            faucet_account_id,
            TRANSFER_AMOUNT * 2,
        )
        .await;
    }
    Ok(())
}

pub async fn test_import_consumed_note_with_proof(client_config: ClientConfig) -> Result<()> {
    let (mut client_1, authenticator_1) = client_config.clone().into_client().await?;
    let (first_regular_account, faucet_account_header) =
        setup_wallet_and_faucet(&mut client_1, AccountStorageMode::Private, &authenticator_1)
            .await?;

    let (mut client_2, authenticator_2) = ClientConfig::default()
        .with_rpc_endpoint(client_config.rpc_endpoint())
        .into_client()
        .await?;
    let (client_2_account, _) =
        insert_new_wallet(&mut client_2, AccountStorageMode::Private, &authenticator_2).await?;

    wait_for_node(&mut client_2).await;

    let from_account_id = first_regular_account.id();
    let to_account_id = client_2_account.id();
    let faucet_account_id = faucet_account_header.id();

    let tx_id =
        mint_and_consume(&mut client_1, from_account_id, faucet_account_id, NoteType::Private)
            .await;
    wait_for_tx(&mut client_1, tx_id).await?;

    let current_block_num = client_1.get_sync_height().await.unwrap();
    let asset = FungibleAsset::new(faucet_account_id, TRANSFER_AMOUNT).unwrap();

    println!("Running P2IDE tx...");
    let tx_request = TransactionRequestBuilder::new().build_pay_to_id(
        PaymentNoteDescription::new(vec![Asset::Fungible(asset)], from_account_id, to_account_id)
            .with_reclaim_height(current_block_num),
        NoteType::Private,
        client_1.rng(),
    )?;
    execute_tx_and_sync(&mut client_1, from_account_id, tx_request).await?;
    let note = client_1
        .get_input_notes(NoteFilter::Committed)
        .await
        .unwrap()
        .first()
        .unwrap()
        .clone();

    // Consume the note with the sender account

    println!("Consuming Note...");
    let tx_request = TransactionRequestBuilder::new().build_consume_notes(vec![note.id()]).unwrap();
    execute_tx_and_sync(&mut client_1, from_account_id, tx_request).await?;

    // Import the consumed note
    client_2
        .import_note(NoteFile::NoteWithProof(
            note.clone().try_into().unwrap(),
            note.inclusion_proof().unwrap().clone(),
        ))
        .await?;

    let consumed_note = client_2.get_input_note(note.id()).await.unwrap().unwrap();
    assert!(matches!(consumed_note.state(), InputNoteState::ConsumedExternal { .. }));
    Ok(())
}

pub async fn test_import_consumed_note_with_id(client_config: ClientConfig) -> Result<()> {
    let (mut client_1, authenticator) = client_config.clone().into_client().await?;
    let (first_regular_account, second_regular_account, faucet_account_header) =
        setup_two_wallets_and_faucet(&mut client_1, AccountStorageMode::Private, &authenticator)
            .await?;

    let (mut client_2, _) = ClientConfig::default()
        .with_rpc_endpoint(client_config.rpc_endpoint())
        .into_client()
        .await?;

    wait_for_node(&mut client_2).await;

    let from_account_id = first_regular_account.id();
    let to_account_id = second_regular_account.id();
    let faucet_account_id = faucet_account_header.id();

    let tx_id =
        mint_and_consume(&mut client_1, from_account_id, faucet_account_id, NoteType::Private)
            .await;
    wait_for_tx(&mut client_1, tx_id).await?;

    let current_block_num = client_1.get_sync_height().await.unwrap();
    let asset = FungibleAsset::new(faucet_account_id, TRANSFER_AMOUNT).unwrap();

    println!("Running P2IDE tx...");
    let tx_request = TransactionRequestBuilder::new().build_pay_to_id(
        PaymentNoteDescription::new(vec![Asset::Fungible(asset)], from_account_id, to_account_id)
            .with_reclaim_height(current_block_num),
        NoteType::Public,
        client_1.rng(),
    )?;
    execute_tx_and_sync(&mut client_1, from_account_id, tx_request).await?;
    let note = client_1
        .get_input_notes(NoteFilter::Committed)
        .await
        .unwrap()
        .first()
        .unwrap()
        .clone();

    // Consume the note with the sender account

    println!("Consuming Note...");
    let tx_request = TransactionRequestBuilder::new().build_consume_notes(vec![note.id()]).unwrap();
    execute_tx_and_sync(&mut client_1, from_account_id, tx_request).await?;
    client_2.sync_state().await.unwrap();

    // Import the consumed note
    client_2.import_note(NoteFile::NoteId(note.id())).await.unwrap();

    let consumed_note = client_2.get_input_note(note.id()).await.unwrap().unwrap();
    assert!(matches!(consumed_note.state(), InputNoteState::ConsumedExternal { .. }));
    Ok(())
}

pub async fn test_import_note_with_proof(client_config: ClientConfig) -> Result<()> {
    let (mut client_1, authenticator) = client_config.clone().into_client().await?;
    let (first_regular_account, second_regular_account, faucet_account_header) =
        setup_two_wallets_and_faucet(&mut client_1, AccountStorageMode::Private, &authenticator)
            .await?;

    let (mut client_2, _) = ClientConfig::default()
        .with_rpc_endpoint(client_config.rpc_endpoint())
        .into_client()
        .await?;

    wait_for_node(&mut client_2).await;

    let from_account_id = first_regular_account.id();
    let to_account_id = second_regular_account.id();
    let faucet_account_id = faucet_account_header.id();

    let tx_id =
        mint_and_consume(&mut client_1, from_account_id, faucet_account_id, NoteType::Private)
            .await;
    wait_for_tx(&mut client_1, tx_id).await?;

    let current_block_num = client_1.get_sync_height().await.unwrap();
    let asset = FungibleAsset::new(faucet_account_id, TRANSFER_AMOUNT).unwrap();

    println!("Running P2IDE tx...");
    let tx_request = TransactionRequestBuilder::new().build_pay_to_id(
        PaymentNoteDescription::new(vec![Asset::Fungible(asset)], from_account_id, to_account_id)
            .with_reclaim_height(current_block_num),
        NoteType::Private,
        client_1.rng(),
    )?;
    execute_tx_and_sync(&mut client_1, from_account_id, tx_request).await?;

    let note = client_1
        .get_input_notes(NoteFilter::Committed)
        .await
        .unwrap()
        .first()
        .unwrap()
        .clone();

    // Import the consumed note
    client_2
        .import_note(NoteFile::NoteWithProof(
            note.clone().try_into().unwrap(),
            note.inclusion_proof().unwrap().clone(),
        ))
        .await?;

    let imported_note = client_2.get_input_note(note.id()).await.unwrap().unwrap();
    assert!(matches!(imported_note.state(), InputNoteState::Unverified { .. }));

    client_2.sync_state().await.unwrap();
    let imported_note = client_2.get_input_note(note.id()).await.unwrap().unwrap();
    assert!(matches!(imported_note.state(), InputNoteState::Committed { .. }));
    Ok(())
}

pub async fn test_discarded_transaction(client_config: ClientConfig) -> Result<()> {
    let (mut client_1, authenticator_1) = client_config.clone().into_client().await?;
    let (first_regular_account, faucet_account_header) =
        setup_wallet_and_faucet(&mut client_1, AccountStorageMode::Private, &authenticator_1)
            .await?;

    let (mut client_2, authenticator_2) = ClientConfig::default()
        .with_rpc_endpoint(client_config.rpc_endpoint())
        .into_client()
        .await?;
    let (second_regular_account, ..) =
        insert_new_wallet(&mut client_2, AccountStorageMode::Private, &authenticator_2).await?;

    wait_for_node(&mut client_2).await;

    let from_account_id = first_regular_account.id();
    let to_account_id = second_regular_account.id();
    let faucet_account_id = faucet_account_header.id();

    let tx_id =
        mint_and_consume(&mut client_1, from_account_id, faucet_account_id, NoteType::Private)
            .await;
    wait_for_tx(&mut client_1, tx_id).await?;

    let current_block_num = client_1.get_sync_height().await.unwrap();
    let asset = FungibleAsset::new(faucet_account_id, TRANSFER_AMOUNT).unwrap();

    println!("Running P2IDE tx...");
    let tx_request = TransactionRequestBuilder::new().build_pay_to_id(
        PaymentNoteDescription::new(vec![Asset::Fungible(asset)], from_account_id, to_account_id)
            .with_reclaim_height(current_block_num),
        NoteType::Public,
        client_1.rng(),
    )?;

    execute_tx_and_sync(&mut client_1, from_account_id, tx_request).await?;
    client_2.sync_state().await.unwrap();
    let note = client_1
        .get_input_notes(NoteFilter::Committed)
        .await
        .unwrap()
        .first()
        .unwrap()
        .clone();

    println!("Consuming Note...");
    let tx_request = TransactionRequestBuilder::new().build_consume_notes(vec![note.id()]).unwrap();

    // Consume the note in client 1 but dont submit it to the node
    let tx_result = client_1.new_transaction(from_account_id, tx_request.clone()).await.unwrap();
    let tx_id = tx_result.executed_transaction().id();
    client_1.testing_prove_transaction(&tx_result).await.unwrap();

    // Store the account state before applying the transaction
    let account_before_tx = client_1.get_account(from_account_id).await.unwrap().unwrap();
    let account_hash_before_tx = account_before_tx.account().commitment();

    // Apply the transaction
    client_1.testing_apply_transaction(tx_result).await.unwrap();

    // Check that the account state has changed after applying the transaction
    let account_after_tx = client_1.get_account(from_account_id).await.unwrap().unwrap();
    let account_hash_after_tx = account_after_tx.account().commitment();

    assert_ne!(
        account_hash_before_tx, account_hash_after_tx,
        "Account hash should change after applying the transaction"
    );

    let note_record = client_1.get_input_note(note.id()).await.unwrap().unwrap();
    assert!(matches!(note_record.state(), InputNoteState::ProcessingAuthenticated(_)));

    // Consume the note in client 2
    execute_tx_and_sync(&mut client_2, to_account_id, tx_request).await?;

    let note_record = client_2.get_input_note(note.id()).await.unwrap().unwrap();
    assert!(matches!(note_record.state(), InputNoteState::ConsumedAuthenticatedLocal(_)));

    // After sync the note in client 1 should be consumed externally and the transaction discarded
    client_1.sync_state().await.unwrap();
    let note_record = client_1.get_input_note(note.id()).await.unwrap().unwrap();
    assert!(matches!(note_record.state(), InputNoteState::ConsumedExternal(_)));
    let tx_record = client_1
        .get_transactions(TransactionFilter::All)
        .await
        .unwrap()
        .into_iter()
        .find(|tx| tx.id == tx_id)
        .with_context(|| {
            format!("Transaction with id {tx_id} not found in discarded transactions")
        })?;
    assert!(matches!(
        tx_record.status,
        TransactionStatus::Discarded(DiscardCause::InputConsumed)
    ));

    // Check that the account state has been rolled back after the transaction was discarded
    let account_after_sync = client_1.get_account(from_account_id).await.unwrap().unwrap();
    let account_hash_after_sync = account_after_sync.account().commitment();

    assert_ne!(
        account_hash_after_sync, account_hash_after_tx,
        "Account hash should change after transaction was discarded"
    );
    assert_eq!(
        account_hash_after_sync, account_hash_before_tx,
        "Account hash should be rolled back to the value before the transaction"
    );
    Ok(())
}

struct AlwaysFailingProver;

impl AlwaysFailingProver {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl TransactionProver for AlwaysFailingProver {
    async fn prove(
        &self,
        _inputs: TransactionInputs,
    ) -> Result<ProvenTransaction, TransactionProverError> {
        return Err(TransactionProverError::other("This prover always fails"));
    }
}

pub async fn test_custom_transaction_prover(client_config: ClientConfig) -> Result<()> {
    let (mut client, authenticator) = client_config.into_client().await?;
    let (first_regular_account, faucet_account_header) =
        setup_wallet_and_faucet(&mut client, AccountStorageMode::Private, &authenticator).await?;

    let from_account_id = first_regular_account.id();
    let faucet_account_id = faucet_account_header.id();

    let fungible_asset = FungibleAsset::new(faucet_account_id, MINT_AMOUNT).unwrap();

    let tx_request = TransactionRequestBuilder::new().build_mint_fungible_asset(
        fungible_asset,
        from_account_id,
        NoteType::Private,
        client.rng(),
    )?;

    let transaction_execution_result =
        client.new_transaction(faucet_account_id, tx_request.clone()).await.unwrap();

    let result = client
        .submit_transaction_with_prover(
            transaction_execution_result,
            Arc::new(AlwaysFailingProver::new()),
        )
        .await;

    assert!(matches!(
        result,
        Err(ClientError::TransactionProvingError(TransactionProverError::Other {
            error_msg: _,
            source: _
        }))
    ));
    Ok(())
}

pub async fn test_locked_account(client_config: ClientConfig) -> Result<()> {
    let (mut client_1, authenticator) = client_config.clone().into_client().await?;

    let (faucet_account, _) =
        insert_new_fungible_faucet(&mut client_1, AccountStorageMode::Private, &authenticator)
            .await?;

    let (private_account, _) =
        insert_new_wallet(&mut client_1, AccountStorageMode::Private, &authenticator).await?;

    let from_account_id = private_account.id();
    let faucet_account_id = faucet_account.id();

    wait_for_node(&mut client_1).await;

    let tx_id =
        mint_and_consume(&mut client_1, from_account_id, faucet_account_id, NoteType::Private)
            .await;
    wait_for_tx(&mut client_1, tx_id).await?;

    let private_account = client_1.get_account(from_account_id).await.unwrap().unwrap().into();

    // Import private account in client 2
    let (mut client_2, _) = ClientConfig::default()
        .with_rpc_endpoint(client_config.rpc_endpoint())
        .into_client()
        .await?;
    client_2.add_account(&private_account, false).await.unwrap();

    wait_for_node(&mut client_2).await;

    // When imported the account shouldn't be locked
    let account_record = client_2.get_account(from_account_id).await.unwrap().unwrap();
    assert!(!account_record.is_locked());

    // Consume note with private account in client 1
    let tx_id =
        mint_and_consume(&mut client_1, from_account_id, faucet_account_id, NoteType::Private)
            .await;
    wait_for_tx(&mut client_1, tx_id).await?;

    // After sync the private account should be locked in client 2
    let summary = client_2.sync_state().await.unwrap();
    assert!(summary.locked_accounts.contains(&from_account_id));
    let account_record = client_2.get_account(from_account_id).await.unwrap().unwrap();
    assert!(account_record.is_locked());

    // Get updated account from client 1 and import it in client 2 with `overwrite` flag
    let updated_private_account =
        client_1.get_account(from_account_id).await.unwrap().unwrap().into();
    client_2.add_account(&updated_private_account, true).await.unwrap();

    // After sync the private account shouldn't be locked in client 2
    client_2.sync_state().await.unwrap();
    let account_record = client_2.get_account(from_account_id).await.unwrap().unwrap();
    assert!(!account_record.is_locked());
    Ok(())
}

pub async fn test_expired_transaction_fails(client_config: ClientConfig) -> Result<()> {
    let (mut client, authenticator) = client_config.into_client().await?;
    let (faucet_account, _) =
        insert_new_fungible_faucet(&mut client, AccountStorageMode::Private, &authenticator)
            .await?;

    let (private_account, ..) =
        insert_new_wallet(&mut client, AccountStorageMode::Private, &authenticator).await?;

    let from_account_id = private_account.id();
    let faucet_account_id = faucet_account.id();

    wait_for_node(&mut client).await;

    let expiration_delta = 2;

    // Create a Mint Tx for 1000 units of our fungible asset
    let fungible_asset = FungibleAsset::new(faucet_account_id, MINT_AMOUNT).unwrap();
    println!("Minting Asset");
    let tx_request = TransactionRequestBuilder::new()
        .expiration_delta(expiration_delta)
        .build_mint_fungible_asset(
            fungible_asset,
            from_account_id,
            NoteType::Public,
            client.rng(),
        )?;

    println!("Executing transaction...");
    let transaction_execution_result =
        client.new_transaction(faucet_account_id, tx_request).await.unwrap();

    println!("Transaction executed successfully");
    wait_for_blocks(&mut client, (expiration_delta + 1).into()).await;

    println!("Sending transaction to node");
    let submitted_tx_result = client.submit_transaction(transaction_execution_result).await;

    assert!(submitted_tx_result.is_err());
    Ok(())
}

/// Tests that RPC methods that are not directly related to the client logic
/// (like GetBlockByNumber) work correctly
pub async fn test_unused_rpc_api(client_config: ClientConfig) -> Result<()> {
    let (mut client, keystore) = client_config.into_client().await?;

    let (first_basic_account, faucet_account) =
        setup_wallet_and_faucet(&mut client, AccountStorageMode::Public, &keystore).await?;

    wait_for_node(&mut client).await;
    client.sync_state().await.unwrap();

    let first_block_num = client.get_sync_height().await.unwrap();

    let (block_header, _) = client
        .test_rpc_api()
        .get_block_header_by_number(Some(first_block_num), false)
        .await?;
    let block = client.test_rpc_api().get_block_by_number(first_block_num).await.unwrap();

    assert_eq!(&block_header, block.header());

    let (tx_id, note) =
        mint_note(&mut client, first_basic_account.id(), faucet_account.id(), NoteType::Public)
            .await;
    wait_for_tx(&mut client, tx_id).await?;

    let tx_id =
        consume_notes(&mut client, first_basic_account.id(), std::slice::from_ref(&note)).await;
    wait_for_tx(&mut client, tx_id).await?;

    // Define the account code for the custom library
    let custom_code = "
        use.miden::account

        export.update_map
            push.1.2.3.4
            # => [VALUE]
            push.0.0.0.0
            # => [KEY, VALUE]
            push.1
            # => [index, KEY, VALUE]
            exec.account::set_map_item
            dropw dropw dropw dropw
        end
    ";

    let mut storage_map = StorageMap::new();
    storage_map.insert(
        [Felt::new(1), Felt::new(2), Felt::new(3), Felt::new(4)].into(),
        [Felt::new(0), Felt::new(0), Felt::new(0), Felt::new(1)].into(),
    )?;

    let storage_slots = vec![StorageSlot::empty_map(), StorageSlot::Map(storage_map)];
    let (account_with_map_item, _) = insert_account_with_custom_component(
        &mut client,
        custom_code,
        storage_slots,
        AccountStorageMode::Public,
        &keystore,
    )
    .await?;

    client.sync_state().await.unwrap();

    let assembler = TransactionKernel::assembler();
    let source_manager = Arc::new(DefaultSourceManager::default());
    let module = Module::parser(ModuleKind::Library)
        .parse_str(
            LibraryPath::new("custom_library::set_map_item_library")
                .context("failed to create library path for custom library")?,
            custom_code,
            &source_manager,
        )
        .unwrap();
    let custom_lib = assembler.assemble_library([module]).unwrap();

    let tx_script = ScriptBuilder::new(true)
        .with_statically_linked_library(&custom_lib)?
        .compile_tx_script(
            "
        use.custom_library::set_map_item_library

        begin
             call.set_map_item_library::update_map
        end
        ",
        )?;

    let tx_request = TransactionRequestBuilder::new().custom_script(tx_script).build()?;
    execute_tx_and_sync(&mut client, account_with_map_item.id(), tx_request.clone()).await?;

    // Mint a new fungible asset to check account vault changes
    let faucet = insert_new_fungible_faucet(&mut client, AccountStorageMode::Private, &keystore)
        .await?
        .0;

    let fungible_asset = FungibleAsset::new(faucet.id(), MINT_AMOUNT)?;
    let tx_request = TransactionRequestBuilder::new().build_mint_fungible_asset(
        fungible_asset,
        first_basic_account.id(),
        NoteType::Public,
        client.rng(),
    )?;
    let note_id = tx_request.expected_output_own_notes().pop().unwrap().id();
    execute_tx_and_sync(&mut client, fungible_asset.faucet_id(), tx_request.clone()).await?;

    let tx_request = TransactionRequestBuilder::new().build_consume_notes(vec![note_id])?;
    execute_tx_and_sync(&mut client, first_basic_account.id(), tx_request).await?;

    let nullifier = note.nullifier();

    let node_nullifier = client
        .test_rpc_api()
        .sync_nullifiers(&[nullifier.prefix()], 0.into(), None)
        .await
        .unwrap()
        .pop()
        .with_context(|| "no nullifier found in sync_nullifiers response")?;
    let node_nullifier_proof = client
        .test_rpc_api()
        .check_nullifiers(&[nullifier])
        .await
        .unwrap()
        .pop()
        .with_context(|| "no nullifier proof returned from check_nullifiers RPC API")?;
    let retrieved_note_script = client
        .test_rpc_api()
        .get_note_script_by_root(note.script().root())
        .await
        .unwrap();
    let sync_storage_maps = client
        .test_rpc_api()
        .sync_storage_maps(0.into(), None, account_with_map_item.id())
        .await
        .unwrap();
    let account_vault_info = client
        .test_rpc_api()
        .sync_account_vault(0.into(), None, first_basic_account.id())
        .await
        .unwrap();
    let transactions_info = client
        .test_rpc_api()
        .sync_transactions(0.into(), None, vec![first_basic_account.id()])
        .await
        .unwrap();

    // Remove debug decorators from original note script, as they are not persisted on submission
    // https://github.com/0xMiden/miden-base/issues/1812
    let mut mast = (*note.script().mast()).clone();
    mast.strip_decorators();
    let note_script = NoteScript::from_parts(Arc::new(mast), note.script().entrypoint());

    assert_eq!(node_nullifier.nullifier, nullifier);
    assert_eq!(node_nullifier_proof.leaf().entries().first().unwrap().0, nullifier.as_word());
    assert_eq!(note_script, retrieved_note_script);
    assert!(!sync_storage_maps.updates.is_empty());
    assert!(!account_vault_info.updates.is_empty());
    assert!(!transactions_info.transaction_records.is_empty());

    Ok(())
}

pub async fn test_ignore_invalid_notes(client_config: ClientConfig) -> Result<()> {
    let (mut client, authenticator) = client_config.into_client().await?;
    let (regular_account, second_regular_account, faucet_account_header) =
        setup_two_wallets_and_faucet(&mut client, AccountStorageMode::Private, &authenticator)
            .await?;

    let account_id = regular_account.id();
    let second_account_id = second_regular_account.id();
    let faucet_account_id = faucet_account_header.id();

    // Mint 2 valid notes
    let (tx_id_1, note_1) =
        mint_note(&mut client, account_id, faucet_account_id, NoteType::Private).await;
    wait_for_tx(&mut client, tx_id_1).await?;
    let (tx_id_2, note_2) =
        mint_note(&mut client, account_id, faucet_account_id, NoteType::Private).await;
    wait_for_tx(&mut client, tx_id_2).await?;

    // Mint 2 invalid notes
    let (tx_id_3, note_3) =
        mint_note(&mut client, second_account_id, faucet_account_id, NoteType::Private).await;
    wait_for_tx(&mut client, tx_id_3).await?;
    let (tx_id_4, note_4) =
        mint_note(&mut client, second_account_id, faucet_account_id, NoteType::Private).await;
    wait_for_tx(&mut client, tx_id_4).await?;

    // Create a transaction to consume all 4 notes but ignore the invalid ones
    let tx_request = TransactionRequestBuilder::new()
        .ignore_invalid_input_notes()
        .build_consume_notes(vec![note_1.id(), note_3.id(), note_2.id(), note_4.id()])?;

    execute_tx_and_sync(&mut client, account_id, tx_request).await?;

    // Check that only the valid notes were consumed
    let consumed_notes = client.get_input_notes(NoteFilter::Consumed).await.unwrap();
    assert_eq!(consumed_notes.len(), 2);
    assert!(consumed_notes.iter().any(|note| note.id() == note_1.id()));
    assert!(consumed_notes.iter().any(|note| note.id() == note_2.id()));
    Ok(())
}

pub async fn test_output_only_note(client_config: ClientConfig) -> Result<()> {
    let (mut client, authenticator) = client_config.into_client().await?;

    let faucet =
        insert_new_fungible_faucet(&mut client, AccountStorageMode::Private, &authenticator)
            .await
            .unwrap()
            .0;

    let fungible_asset = FungibleAsset::new(faucet.id(), MINT_AMOUNT).unwrap();
    let tx_request = TransactionRequestBuilder::new().build_mint_fungible_asset(
        fungible_asset,
        AccountId::try_from(ACCOUNT_ID_REGULAR).unwrap(),
        NoteType::Public,
        client.rng(),
    )?;
    let note_id = tx_request.expected_output_own_notes().pop().unwrap().id();
    execute_tx_and_sync(&mut client, fungible_asset.faucet_id(), tx_request.clone()).await?;

    // The created note should be an output only note because it is not consumable by any client
    // account.
    let input_note = client.get_input_note(note_id).await.unwrap();
    assert!(input_note.is_none());

    let output_note = client.get_output_note(note_id).await.unwrap();
    assert!(output_note.is_some());
    Ok(())
}
