use std::boxed::Box;
use std::collections::BTreeSet;
use std::env::temp_dir;
use std::println;
use std::string::ToString;
use std::sync::Arc;

use miden_client::account::{AccountIdAddress, Address, AddressInterface};
use miden_client::builder::ClientBuilder;
use miden_client::keystore::FilesystemKeyStore;
use miden_client::note::{BlockNumber, NoteRelevance};
use miden_client::rpc::NodeRpcClient;
use miden_client::store::input_note_states::ConsumedAuthenticatedLocalNoteState;
use miden_client::store::{InputNoteRecord, InputNoteState, NoteFilter, TransactionFilter};
use miden_client::sync::NoteTagSource;
use miden_client::testing::account_id::ACCOUNT_ID_PUBLIC_NON_FUNGIBLE_FAUCET;
use miden_client::testing::common::{
    ACCOUNT_ID_REGULAR,
    MINT_AMOUNT,
    RECALL_HEIGHT_DELTA,
    TRANSFER_AMOUNT,
    TestClient,
    TestClientKeyStore,
    assert_account_has_single_asset,
    assert_note_cannot_be_consumed_twice,
    consume_notes,
    create_test_store_path,
    execute_failing_tx,
    mint_and_consume,
    mint_note,
    setup_two_wallets_and_faucet,
    setup_wallet_and_faucet,
};
use miden_client::testing::mock::{MockClient, MockRpcApi};
use miden_client::transaction::{
    DiscardCause,
    PaymentNoteDescription,
    SwapTransactionData,
    TransactionExecutorError,
    TransactionRequestBuilder,
    TransactionRequestError,
    TransactionStatus,
};
use miden_client::{ClientError, DebugMode};
use miden_client_sqlite_store::ClientBuilderSqliteExt;
use miden_lib::account::auth::AuthRpoFalcon512;
use miden_lib::account::faucets::BasicFungibleFaucet;
use miden_lib::account::interface::AccountInterfaceError;
use miden_lib::account::wallets::BasicWallet;
use miden_lib::note::utils;
use miden_lib::note::well_known_note::WellKnownNote;
use miden_lib::testing::mock_account::MockAccountExt;
use miden_lib::testing::note::NoteBuilder;
use miden_lib::transaction::TransactionKernel;
use miden_lib::utils::{Deserializable, ScriptBuilder, Serializable};
use miden_objects::account::{
    Account,
    AccountBuilder,
    AccountCode,
    AccountComponent,
    AccountHeader,
    AccountId,
    AccountStorageMode,
    AccountType,
    AuthSecretKey,
    PublicKeyCommitment,
    StorageMap,
    StorageSlot,
};
use miden_objects::assembly::{Assembler, DefaultSourceManager, LibraryPath, Module, ModuleKind};
use miden_objects::asset::{Asset, AssetWitness, FungibleAsset, TokenSymbol};
use miden_objects::crypto::dsa::rpo_falcon512::SecretKey;
use miden_objects::crypto::rand::{FeltRng, RpoRandomCoin};
use miden_objects::note::{
    Note,
    NoteAssets,
    NoteExecutionHint,
    NoteExecutionMode,
    NoteFile,
    NoteInputs,
    NoteMetadata,
    NoteRecipient,
    NoteTag,
    NoteType,
};
use miden_objects::testing::account_id::{
    ACCOUNT_ID_PRIVATE_SENDER,
    ACCOUNT_ID_PUBLIC_FUNGIBLE_FAUCET_1,
    ACCOUNT_ID_PUBLIC_FUNGIBLE_FAUCET_2,
    ACCOUNT_ID_REGULAR_PRIVATE_ACCOUNT_UPDATABLE_CODE,
    ACCOUNT_ID_REGULAR_PUBLIC_ACCOUNT_IMMUTABLE_CODE,
    ACCOUNT_ID_REGULAR_PUBLIC_ACCOUNT_UPDATABLE_CODE,
};
use miden_objects::transaction::{InputNote, OutputNote};
use miden_objects::vm::AdviceInputs;
use miden_objects::{EMPTY_WORD, Felt, ONE, Word, ZERO};
use miden_testing::{MockChain, MockChainBuilder, TxContextInput};
use rand::rngs::StdRng;
use rand::{Rng, RngCore};

pub mod store;
mod transaction;
mod transport;

/// Constant that represents the number of blocks until the transaction is considered
/// stale.
const TX_GRACEFUL_BLOCKS: u32 = 20;

// TESTS
// ================================================================================================

#[tokio::test]
async fn input_notes_round_trip() {
    // generate test client with a random store name
    let (mut client, rpc_api, keystore) = Box::pin(create_test_client()).await;

    insert_new_wallet(&mut client, AccountStorageMode::Private, &keystore)
        .await
        .unwrap();
    // generate test data
    let available_notes = rpc_api.get_public_available_notes();

    // insert notes into database
    for note in &available_notes {
        client
            .import_note(NoteFile::NoteWithProof(
                note.note().unwrap().clone(),
                note.inclusion_proof().clone(),
            ))
            .await
            .unwrap();
    }

    // retrieve notes from database
    assert_eq!(client.get_input_notes(NoteFilter::Unverified).await.unwrap().len(), 1);
    // NOTE: the following asserts involve the spawn notes
    assert_eq!(client.get_input_notes(NoteFilter::Consumed).await.unwrap().len(), 3);
    let retrieved_notes = client.get_input_notes(NoteFilter::All).await.unwrap();
    assert_eq!(retrieved_notes.len(), 4);

    let recorded_notes: Vec<InputNoteRecord> = available_notes
        .into_iter()
        .map(|n| {
            let input_note: InputNote = n.try_into().unwrap();
            input_note.into()
        })
        .collect();
    // compare notes
    for (recorded_note, retrieved_note) in recorded_notes.iter().zip(retrieved_notes) {
        assert_eq!(recorded_note.id(), retrieved_note.id());
    }
}

#[tokio::test]
async fn get_input_note() {
    // generate test client with a random store name
    let (mut client, rpc_api, _) = Box::pin(create_test_client()).await;
    // Get note from mocked RPC backend since any note works here
    let original_note = rpc_api.get_available_notes()[0].note().unwrap().clone();

    // insert Note into database
    let note: InputNoteRecord = original_note.clone().into();
    client
        .import_note(NoteFile::NoteDetails {
            details: note.into(),
            tag: None,
            after_block_num: 0.into(),
        })
        .await
        .unwrap();

    // retrieve note from database
    let retrieved_note = client.get_input_note(original_note.id()).await.unwrap().unwrap();

    let recorded_note: InputNoteRecord = original_note.into();
    assert_eq!(recorded_note.id(), retrieved_note.id());
}

#[tokio::test]
async fn insert_basic_account() {
    // generate test client with a random store name
    let (mut client, _rpc_api, keystore) = Box::pin(create_test_client()).await;

    // Insert Account
    let account_insert_result =
        insert_new_wallet(&mut client, AccountStorageMode::Private, &keystore).await;
    assert!(account_insert_result.is_ok());

    let account = account_insert_result.unwrap();

    // Fetch Account
    let fetched_account_data = client.get_account(account.id()).await;
    assert!(fetched_account_data.is_ok());

    let fetched_account = fetched_account_data.unwrap().unwrap();
    let fetched_account_seed = fetched_account.seed();
    let fetched_account: Account = fetched_account.into();

    // Validate header has matching data
    assert_eq!(account.id(), fetched_account.id());
    assert_eq!(account.nonce(), fetched_account.nonce());
    assert_eq!(account.vault(), fetched_account.vault());
    assert_eq!(account.storage().commitment(), fetched_account.storage().commitment());
    assert_eq!(account.code().commitment(), fetched_account.code().commitment());

    // Validate seed matches
    assert_eq!(account.seed(), fetched_account_seed);
}

#[tokio::test]
async fn insert_faucet_account() {
    // generate test client with a random store name
    let (mut client, _rpc_api, keystore) = Box::pin(create_test_client()).await;

    // Insert Account
    let account_insert_result =
        insert_new_fungible_faucet(&mut client, AccountStorageMode::Private, &keystore).await;
    assert!(account_insert_result.is_ok());

    let account = account_insert_result.unwrap();
    let account_seed = account.seed().expect("newly built account should always contain a seed");

    // Fetch Account
    let fetched_account_data = client.get_account(account.id()).await;
    assert!(fetched_account_data.is_ok());

    let fetched_account = fetched_account_data.unwrap().unwrap();
    let fetched_account_seed = fetched_account.seed();
    let fetched_account: Account = fetched_account.into();

    // Validate header has matching data
    assert_eq!(account.id(), fetched_account.id());
    assert_eq!(account.nonce(), fetched_account.nonce());
    assert_eq!(account.vault(), fetched_account.vault());
    assert_eq!(account.storage(), fetched_account.storage());
    assert_eq!(account.code().commitment(), fetched_account.code().commitment());

    // Validate seed matches
    assert_eq!(account_seed, fetched_account_seed.unwrap());
}

#[tokio::test]
async fn insert_same_account_twice_fails() {
    // generate test client with a random store name
    let (mut client, _rpc_api, _) = Box::pin(create_test_client()).await;

    let account = Account::mock(
        ACCOUNT_ID_PUBLIC_FUNGIBLE_FAUCET_2,
        AuthRpoFalcon512::new(PublicKeyCommitment::from(EMPTY_WORD)),
    );

    assert!(client.add_account(&account, false).await.is_ok());
    assert!(client.add_account(&account, false).await.is_err());
}

#[tokio::test]
async fn account_code() {
    // generate test client with a random store name
    let (mut client, _rpc_api, _) = Box::pin(create_test_client()).await;

    let account = Account::mock(
        ACCOUNT_ID_REGULAR_PRIVATE_ACCOUNT_UPDATABLE_CODE,
        AuthRpoFalcon512::new(PublicKeyCommitment::from(EMPTY_WORD)),
    );

    let account_code = account.code();

    let account_code_bytes = account_code.to_bytes();

    let reconstructed_code = AccountCode::read_from_bytes(&account_code_bytes).unwrap();
    assert_eq!(*account_code, reconstructed_code);

    client.add_account(&account, false).await.unwrap();
    let retrieved_acc = client.get_account(account.id()).await.unwrap().unwrap();
    assert_eq!(*account.code(), *retrieved_acc.account().code());
}

#[tokio::test]
async fn get_account_by_id() {
    // generate test client with a random store name
    let (mut client, _rpc_api, _) = Box::pin(create_test_client()).await;

    let account = Account::mock(
        ACCOUNT_ID_REGULAR_PUBLIC_ACCOUNT_UPDATABLE_CODE,
        AuthRpoFalcon512::new(PublicKeyCommitment::from(EMPTY_WORD)),
    );

    client.add_account(&account, false).await.unwrap();

    // Retrieving an existing account should succeed
    let (acc_from_db, _account_seed) = match client.get_account_header_by_id(account.id()).await {
        Ok(account) => account.unwrap(),
        Err(err) => panic!("Error retrieving account: {err}"),
    };
    assert_eq!(AccountHeader::from(account), acc_from_db);

    // Retrieving a non existing account should fail
    let invalid_id = AccountId::try_from(ACCOUNT_ID_PUBLIC_FUNGIBLE_FAUCET_2).unwrap();
    assert!(client.get_account_header_by_id(invalid_id).await.unwrap().is_none());
}

#[tokio::test]
async fn sync_state() {
    // generate test client with a random store name
    let (mut client, rpc_api, _) = Box::pin(create_test_client()).await;

    // Import first mockchain note as expected
    let expected_notes = rpc_api
        .get_available_notes()
        .into_iter()
        .filter(|n| n.inclusion_proof().location().block_num() != BlockNumber::GENESIS)
        .map(|n| n.note().unwrap().clone())
        .collect::<Vec<Note>>();

    for note in &expected_notes {
        client
            .import_note(NoteFile::NoteDetails {
                details: note.clone().into(),
                after_block_num: 0.into(),
                tag: Some(note.metadata().tag()),
            })
            .await
            .unwrap();
    }

    // assert that we have no consumed nor expected notes prior to syncing state
    assert_eq!(client.get_input_notes(NoteFilter::Consumed).await.unwrap().len(), 0);
    assert_eq!(
        client.get_input_notes(NoteFilter::Expected).await.unwrap().len(),
        expected_notes.len()
    );
    assert_eq!(client.get_input_notes(NoteFilter::Committed).await.unwrap().len(), 0);

    // sync state
    let sync_details = client.sync_state().await.unwrap();

    // verify that the client is synced to the latest block
    assert_eq!(sync_details.block_num, rpc_api.get_chain_tip_block_num());

    // verify that we now have one committed note after syncing state
    assert_eq!(client.get_input_notes(NoteFilter::Committed).await.unwrap().len(), 1);
    assert_eq!(client.get_input_notes(NoteFilter::Consumed).await.unwrap().len(), 1);
    assert_eq!(sync_details.consumed_notes.len(), 1);

    // verify that the latest block number has been updated
    assert_eq!(client.get_sync_height().await.unwrap(), rpc_api.get_chain_tip_block_num());
}

#[tokio::test]
async fn sync_state_mmr() {
    // generate test client with a random store name
    let (mut client, rpc_api, keystore) = Box::pin(create_test_client()).await;
    // Import note and create wallet so that synced notes do not get discarded (due to being
    // irrelevant)
    insert_new_wallet(&mut client, AccountStorageMode::Private, &keystore)
        .await
        .unwrap();

    // Import only public notes
    let notes = rpc_api
        .get_public_available_notes()
        .into_iter()
        .filter_map(|n| n.note().cloned())
        .collect::<Vec<Note>>();

    for note in &notes {
        client
            .import_note(NoteFile::NoteDetails {
                details: note.clone().into(),
                after_block_num: 0.into(),
                tag: Some(note.metadata().tag()),
            })
            .await
            .unwrap();
    }

    // sync state
    let sync_details = client.sync_state().await.unwrap();

    // verify that the client is synced to the latest block
    assert_eq!(sync_details.block_num, rpc_api.get_chain_tip_block_num());

    // verify that the latest block number has been updated
    assert_eq!(client.get_sync_height().await.unwrap(), rpc_api.get_chain_tip_block_num());

    // verify that we inserted the latest block into the DB via the client
    let latest_block = client.get_sync_height().await.unwrap();
    assert_eq!(sync_details.block_num, latest_block);
    assert_eq!(
        rpc_api.get_block_header_by_number(None, false).await.unwrap().0.commitment(),
        client
            .test_store()
            .get_block_headers(&[latest_block].into_iter().collect())
            .await
            .unwrap()[0]
            .0
            .commitment()
    );

    // Try reconstructing the partial_mmr from what's in the database
    let partial_mmr = client.test_store().get_current_partial_mmr().await.unwrap();
    assert!(partial_mmr.forest().num_leaves() >= 6);
    assert!(partial_mmr.open(0).unwrap().is_none());
    assert!(partial_mmr.open(1).unwrap().is_some());
    assert!(partial_mmr.open(2).unwrap().is_none());
    assert!(partial_mmr.open(3).unwrap().is_none());
    assert!(partial_mmr.open(4).unwrap().is_some());
    assert!(partial_mmr.open(5).unwrap().is_none());

    // // Ensure the proofs are valid
    let mmr_proof = partial_mmr.open(1).unwrap().unwrap();
    let (block_1, _) = rpc_api.get_block_header_by_number(Some(1.into()), false).await.unwrap();
    partial_mmr.peaks().verify(block_1.commitment(), mmr_proof).unwrap();

    let mmr_proof = partial_mmr.open(4).unwrap().unwrap();
    let (block_4, _) = rpc_api.get_block_header_by_number(Some(4.into()), false).await.unwrap();
    partial_mmr.peaks().verify(block_4.commitment(), mmr_proof).unwrap();

    // the blocks for both notes should be stored as they are relevant for the client's accounts
    assert_eq!(client.test_store().get_tracked_block_headers().await.unwrap().len(), 2);
}

#[tokio::test]
async fn sync_state_tags() {
    // generate test client with a random store name
    let (mut client, rpc_api, _) = Box::pin(create_test_client()).await;

    // Import first mockchain note as expected
    let expected_notes = rpc_api.get_available_notes();
    for tag in expected_notes.iter().map(|n| n.metadata().tag()) {
        client.add_note_tag(tag).await.unwrap();
    }

    // assert that we have no expected notes prior to syncing state
    assert!(client.get_input_notes(NoteFilter::Expected).await.unwrap().is_empty());

    // sync state
    // The mockchain API has one public note and one private note, so in the end we will have
    // the public one in the client
    let sync_details = client.sync_state().await.unwrap();

    // verify that the client is synced to the latest block
    assert_eq!(
        sync_details.block_num,
        rpc_api.get_block_header_by_number(None, false).await.unwrap().0.block_num()
    );

    // as we are syncing with tags, the response should contain blocks for both notes
    assert_eq!(client.get_input_notes(NoteFilter::All).await.unwrap().len(), 2);
    assert_eq!(client.test_store().get_tracked_block_headers().await.unwrap().len(), 2);
}

#[tokio::test]
async fn tags() {
    // generate test client with a random store name
    let (mut client, _rpc_api, _) = Box::pin(create_test_client()).await;

    // Assert that the store gets created with the tag 0 (used for notes consumable by any account)
    assert!(client.get_note_tags().await.unwrap().is_empty());

    // add a tag
    let tag_1: NoteTag = 1.into();
    let tag_2: NoteTag = 2.into();
    client.add_note_tag(tag_1).await.unwrap();
    client.add_note_tag(tag_2).await.unwrap();

    // verify that the tag is being tracked
    assert_eq!(client.get_note_tags().await.unwrap(), vec![tag_1, tag_2]);

    // attempt to add the same tag again
    client.add_note_tag(tag_1).await.unwrap();

    // verify that the tag is still being tracked only once
    assert_eq!(client.get_note_tags().await.unwrap(), vec![tag_1, tag_2]);

    // Try removing non-existent tag
    let tag_4: NoteTag = 4.into();
    client.remove_note_tag(tag_4).await.unwrap();

    // verify that the tracked tags are unchanged
    assert_eq!(client.get_note_tags().await.unwrap(), vec![tag_1, tag_2]);

    // remove second tag
    client.remove_note_tag(tag_1).await.unwrap();

    // verify that tag_1 is not tracked anymore
    assert_eq!(client.get_note_tags().await.unwrap(), vec![tag_2]);
}

#[tokio::test]
async fn mint_transaction() {
    // generate test client with a random store name
    let (mut client, _rpc_api, keystore) = Box::pin(create_test_client()).await;

    // Faucet account generation
    let faucet = insert_new_fungible_faucet(&mut client, AccountStorageMode::Private, &keystore)
        .await
        .unwrap();

    client.sync_state().await.unwrap();

    // Test submitting a mint transaction
    let transaction_request = TransactionRequestBuilder::new()
        .build_mint_fungible_asset(
            FungibleAsset::new(faucet.id(), 5u64).unwrap(),
            AccountId::try_from(ACCOUNT_ID_PUBLIC_FUNGIBLE_FAUCET_1).unwrap(),
            miden_objects::note::NoteType::Private,
            client.rng(),
        )
        .unwrap();

    let transaction_result =
        Box::pin(client.execute_transaction(faucet.id(), transaction_request.clone()))
            .await
            .unwrap();
    let executed_tx = transaction_result.executed_transaction().clone();

    assert_eq!(executed_tx.account_delta().nonce_delta(), ONE);
}

#[tokio::test]
async fn import_note_validation() {
    // generate test client
    let (mut client, rpc_api, _) = Box::pin(create_test_client()).await;

    // generate test data
    let expected_note = rpc_api.get_available_notes()[0].clone();
    let consumed_note = rpc_api.get_available_notes()[1].clone();

    client
        .import_note(NoteFile::NoteWithProof(
            consumed_note.note().unwrap().clone(),
            consumed_note.inclusion_proof().clone(),
        ))
        .await
        .unwrap();

    client
        .import_note(NoteFile::NoteDetails {
            details: expected_note.note().unwrap().into(),
            after_block_num: 0.into(),
            tag: None,
        })
        .await
        .unwrap();

    let expected_note = Box::pin(client.get_input_note(expected_note.note().unwrap().id()))
        .await
        .unwrap()
        .unwrap();

    let consumed_note = client
        .get_input_note(consumed_note.note().unwrap().id())
        .await
        .unwrap()
        .unwrap();

    assert!(expected_note.inclusion_proof().is_none());
    assert!(consumed_note.is_consumed());
}

#[tokio::test]
async fn transaction_request_expiration() {
    let (mut client, _, keystore) = Box::pin(create_test_client()).await;
    client.sync_state().await.unwrap();

    let current_height = client.get_sync_height().await.unwrap();
    let faucet = insert_new_fungible_faucet(&mut client, AccountStorageMode::Private, &keystore)
        .await
        .unwrap();

    let transaction_request = TransactionRequestBuilder::new()
        .expiration_delta(5)
        .build_mint_fungible_asset(
            FungibleAsset::new(faucet.id(), 5u64).unwrap(),
            AccountId::try_from(ACCOUNT_ID_REGULAR_PUBLIC_ACCOUNT_IMMUTABLE_CODE).unwrap(),
            miden_objects::note::NoteType::Private,
            client.rng(),
        )
        .unwrap();

    let transaction_result =
        Box::pin(client.execute_transaction(faucet.id(), transaction_request.clone()))
            .await
            .unwrap();

    let (_, tx_outputs, ..) = transaction_result.executed_transaction().clone().into_parts();

    assert_eq!(tx_outputs.expiration_block_num, current_height + 5);
}

#[tokio::test]
async fn import_processing_note_returns_error() {
    // generate test client with a random store name
    let (mut client, _rpc_api, keystore) = Box::pin(create_test_client()).await;
    client.sync_state().await.unwrap();

    let account = insert_new_wallet(&mut client, AccountStorageMode::Private, &keystore)
        .await
        .unwrap();

    // Faucet account generation
    let faucet = insert_new_fungible_faucet(&mut client, AccountStorageMode::Private, &keystore)
        .await
        .unwrap();

    // Test submitting a mint transaction
    let transaction_request = TransactionRequestBuilder::new()
        .build_mint_fungible_asset(
            FungibleAsset::new(faucet.id(), 5u64).unwrap(),
            account.id(),
            miden_objects::note::NoteType::Public,
            client.rng(),
        )
        .unwrap();

    Box::pin(client.submit_new_transaction(faucet.id(), transaction_request.clone()))
        .await
        .unwrap();

    let note_id = transaction_request.expected_output_own_notes().pop().unwrap().id();
    let note = client.get_input_note(note_id).await.unwrap().unwrap();

    let input = [(note.try_into().unwrap(), None)];
    let consume_note_request = TransactionRequestBuilder::new()
        .unauthenticated_input_notes(input)
        .build()
        .unwrap();
    Box::pin(client.submit_new_transaction(account.id(), consume_note_request))
        .await
        .unwrap();

    let processing_notes = client.get_input_notes(NoteFilter::Processing).await.unwrap();

    assert!(matches!(
        client
            .import_note(NoteFile::NoteId(processing_notes[0].id()))
            .await
            .unwrap_err(),
        ClientError::NoteImportError { .. }
    ));
}

#[tokio::test]
async fn note_without_asset() {
    let (mut client, _rpc_api, keystore) = Box::pin(create_test_client()).await;

    let faucet = insert_new_fungible_faucet(&mut client, AccountStorageMode::Private, &keystore)
        .await
        .unwrap();

    let wallet = insert_new_wallet(&mut client, AccountStorageMode::Private, &keystore)
        .await
        .unwrap();

    client.sync_state().await.unwrap();

    // Create note without assets
    let serial_num = client.rng().draw_word();
    let recipient = utils::build_p2id_recipient(wallet.id(), serial_num).unwrap();
    let tag = NoteTag::from_account_id(wallet.id());
    let metadata =
        NoteMetadata::new(wallet.id(), NoteType::Private, tag, NoteExecutionHint::always(), ZERO)
            .unwrap();
    let vault = NoteAssets::new(vec![]).unwrap();

    let note = Note::new(vault.clone(), metadata, recipient.clone());

    // Create and execute transaction
    let transaction_request = TransactionRequestBuilder::new()
        .own_output_notes(vec![OutputNote::Full(note)])
        .build()
        .unwrap();

    let transaction =
        Box::pin(client.execute_transaction(wallet.id(), transaction_request.clone())).await;

    assert!(transaction.is_ok());

    // Create the same transaction for the faucet
    let metadata =
        NoteMetadata::new(faucet.id(), NoteType::Private, tag, NoteExecutionHint::always(), ZERO)
            .unwrap();
    let note = Note::new(vault, metadata, recipient);

    let transaction_request = TransactionRequestBuilder::new()
        .own_output_notes(vec![OutputNote::Full(note)])
        .build()
        .unwrap();

    let error = Box::pin(client.submit_new_transaction(faucet.id(), transaction_request))
        .await
        .unwrap_err();

    assert!(matches!(
        error,
        ClientError::TransactionRequestError(TransactionRequestError::AccountInterfaceError(
            AccountInterfaceError::FaucetNoteWithoutAsset
        ))
    ));

    let error = TransactionRequestBuilder::new()
        .build_pay_to_id(
            PaymentNoteDescription::new(vec![], faucet.id(), wallet.id()),
            NoteType::Public,
            client.rng(),
        )
        .unwrap_err();

    assert!(matches!(error, TransactionRequestError::P2IDNoteWithoutAsset));

    let error = TransactionRequestBuilder::new()
        .build_pay_to_id(
            PaymentNoteDescription::new(
                vec![Asset::Fungible(FungibleAsset::new(faucet.id(), 0).unwrap())],
                faucet.id(),
                wallet.id(),
            ),
            NoteType::Public,
            client.rng(),
        )
        .unwrap_err();

    assert!(matches!(error, TransactionRequestError::P2IDNoteWithoutAsset));
}

#[tokio::test]
async fn execute_program() {
    let (mut client, _, keystore) = Box::pin(create_test_client()).await;
    let _ = client.sync_state().await.unwrap();

    let wallet = insert_new_wallet(&mut client, AccountStorageMode::Private, &keystore)
        .await
        .unwrap();

    let code = "
        use.std::sys

        begin
            push.16
            repeat.16
                dup push.1 sub
            end
            exec.sys::truncate_stack
        end
        ";

    let tx_script = client.script_builder().compile_tx_script(code).unwrap();

    let output_stack = Box::pin(client.execute_program(
        wallet.id(),
        tx_script,
        AdviceInputs::default(),
        BTreeSet::new(),
    ))
    .await
    .unwrap();

    let mut expected_stack = [Felt::new(0); 16];
    for (i, element) in expected_stack.iter_mut().enumerate() {
        *element = Felt::new(i as u64);
    }

    assert_eq!(output_stack, expected_stack);
}

#[tokio::test]
async fn real_note_roundtrip() {
    let (mut client, mock_rpc_api, keystore) = Box::pin(create_test_client()).await;
    let wallet = insert_new_wallet(&mut client, AccountStorageMode::Private, &keystore)
        .await
        .unwrap();
    let faucet = insert_new_fungible_faucet(&mut client, AccountStorageMode::Private, &keystore)
        .await
        .unwrap();

    mock_rpc_api.prove_block();
    client.sync_state().await.unwrap();

    // Test submitting a mint transaction
    let transaction_request = TransactionRequestBuilder::new()
        .build_mint_fungible_asset(
            FungibleAsset::new(faucet.id(), 5u64).unwrap(),
            wallet.id(),
            miden_objects::note::NoteType::Public,
            client.rng(),
        )
        .unwrap();

    let note_id = transaction_request.expected_output_own_notes().pop().unwrap().id();
    Box::pin(client.submit_new_transaction(faucet.id(), transaction_request))
        .await
        .unwrap();

    let note = client.get_input_note(note_id).await.unwrap().unwrap();
    assert!(matches!(note.state(), &InputNoteState::Expected(_)));

    mock_rpc_api.prove_block();
    client.sync_state().await.unwrap();

    let note = client.get_input_note(note_id).await.unwrap().unwrap();
    assert!(matches!(note.state(), &InputNoteState::Committed(_)));

    // Consume note
    let transaction_request =
        TransactionRequestBuilder::new().build_consume_notes(vec![note_id]).unwrap();

    Box::pin(client.submit_new_transaction(wallet.id(), transaction_request))
        .await
        .unwrap();

    mock_rpc_api.prove_block();
    client.sync_state().await.unwrap();

    let note = client.get_input_note(note_id).await.unwrap().unwrap();
    assert!(matches!(note.state(), &InputNoteState::ConsumedAuthenticatedLocal(_)));
}

#[tokio::test]
async fn added_notes() {
    let (mut client, mock_rpc_api, authenticator) = Box::pin(create_test_client()).await;

    let faucet_account_header =
        insert_new_fungible_faucet(&mut client, AccountStorageMode::Private, &authenticator)
            .await
            .unwrap();

    // Mint some asset for an account not tracked by the client. It should not be stored as an
    // input note afterwards since it is not being tracked by the client
    let fungible_asset = FungibleAsset::new(faucet_account_header.id(), MINT_AMOUNT).unwrap();
    let tx_request = TransactionRequestBuilder::new()
        .build_mint_fungible_asset(
            fungible_asset,
            AccountId::try_from(ACCOUNT_ID_REGULAR).unwrap(),
            NoteType::Private,
            client.rng(),
        )
        .unwrap();
    println!("Running Mint tx...");
    Box::pin(client.submit_new_transaction(faucet_account_header.id(), tx_request))
        .await
        .unwrap();
    mock_rpc_api.prove_block();
    client.sync_state().await.unwrap();

    // Check that no new notes were added
    println!("Fetching Committed Notes...");
    let notes = client.get_input_notes(NoteFilter::Committed).await.unwrap();
    assert!(notes.is_empty());
}

#[tokio::test]
async fn p2id_transfer() {
    let (mut client, mock_rpc_api, authenticator) = Box::pin(create_test_client()).await;

    let (first_regular_account, second_regular_account, faucet_account_header) =
        setup_two_wallets_and_faucet(&mut client, AccountStorageMode::Private, &authenticator)
            .await
            .unwrap();

    let from_account_id = first_regular_account.id();
    let to_account_id = second_regular_account.id();
    let faucet_account_id = faucet_account_header.id();

    // First Mint necessary token
    mint_and_consume(&mut client, from_account_id, faucet_account_id, NoteType::Private).await;
    mock_rpc_api.prove_block();
    client.sync_state().await.unwrap();

    assert_account_has_single_asset(&client, from_account_id, faucet_account_id, MINT_AMOUNT).await;

    // Do a transfer from first account to second account
    let asset = FungibleAsset::new(faucet_account_id, TRANSFER_AMOUNT).unwrap();
    println!("Running P2ID tx...");
    let tx_request = TransactionRequestBuilder::new()
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

    let note = tx_request.expected_output_own_notes().pop().unwrap();
    Box::pin(client.submit_new_transaction(from_account_id, tx_request))
        .await
        .unwrap();

    // Check that a note tag started being tracked for this note.
    assert!(
        client
            .get_note_tags()
            .await
            .unwrap()
            .into_iter()
            .any(|tag| tag.source == NoteTagSource::Note(note.id()))
    );

    mock_rpc_api.prove_block();
    client.sync_state().await.unwrap();

    // Check that the tag is not longer being tracked
    assert!(
        !client
            .get_note_tags()
            .await
            .unwrap()
            .into_iter()
            .any(|tag| tag.source == NoteTagSource::Note(note.id()))
    );

    // Check that note is committed for the second account to consume
    println!("Fetching Committed Notes...");
    let notes = client.get_input_notes(NoteFilter::Committed).await.unwrap();
    assert!(!notes.is_empty());

    // Consume P2ID note
    println!("Consuming Note...");
    let tx_request = TransactionRequestBuilder::new()
        .build_consume_notes(vec![notes[0].id()])
        .unwrap();
    Box::pin(client.submit_new_transaction(to_account_id, tx_request))
        .await
        .unwrap();
    mock_rpc_api.prove_block();
    client.sync_state().await.unwrap();

    // Ensure we have nothing else to consume
    let current_notes = client.get_input_notes(NoteFilter::Committed).await.unwrap();
    assert!(current_notes.is_empty());

    let regular_account = client.get_account(from_account_id).await.unwrap().unwrap();
    let seed = regular_account.seed();
    let regular_account: Account = regular_account.into();

    // The seed should not be retrieved due to the account not being new
    assert!(!regular_account.is_new() && seed.is_none());
    assert_eq!(regular_account.vault().assets().count(), 1);
    let asset = regular_account.vault().assets().next().unwrap();

    // Validate the transferred amounts
    if let Asset::Fungible(fungible_asset) = asset {
        assert_eq!(fungible_asset.amount(), MINT_AMOUNT - TRANSFER_AMOUNT);
    } else {
        panic!("Error: Account should have a fungible asset");
    }

    let regular_account: Account = client.get_account(to_account_id).await.unwrap().unwrap().into();
    assert_eq!(regular_account.vault().assets().count(), 1);
    let asset = regular_account.vault().assets().next().unwrap();

    if let Asset::Fungible(fungible_asset) = asset {
        assert_eq!(fungible_asset.amount(), TRANSFER_AMOUNT);
    } else {
        panic!("Error: Account should have a fungible asset");
    }

    assert_note_cannot_be_consumed_twice(&mut client, to_account_id, notes[0].id()).await;
}

#[tokio::test]
async fn p2id_transfer_failing_not_enough_balance() {
    let (mut client, mock_rpc_api, authenticator) = Box::pin(create_test_client()).await;

    let (first_regular_account, second_regular_account, faucet_account_header) =
        setup_two_wallets_and_faucet(&mut client, AccountStorageMode::Private, &authenticator)
            .await
            .unwrap();

    let from_account_id = first_regular_account.id();
    let to_account_id = second_regular_account.id();
    let faucet_account_id = faucet_account_header.id();

    // First Mint necessary token
    mint_and_consume(&mut client, from_account_id, faucet_account_id, NoteType::Private).await;
    mock_rpc_api.prove_block();
    client.sync_state().await.unwrap();

    // Do a transfer from first account to second account
    let asset = FungibleAsset::new(faucet_account_id, MINT_AMOUNT + 1).unwrap();
    println!("Running P2ID tx...");
    let tx_request = TransactionRequestBuilder::new()
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
    execute_failing_tx(
        &mut client,
        from_account_id,
        tx_request,
        ClientError::AssetError(miden_objects::AssetError::FungibleAssetAmountNotSufficient {
            minuend: MINT_AMOUNT,
            subtrahend: MINT_AMOUNT + 1,
        }),
    )
    .await;
}

#[tokio::test]
async fn p2ide_transfer_consumed_by_target() {
    let (mut client, mock_rpc_api, authenticator) = Box::pin(create_test_client()).await;

    let (first_regular_account, second_regular_account, faucet_account_header) =
        setup_two_wallets_and_faucet(&mut client, AccountStorageMode::Private, &authenticator)
            .await
            .unwrap();

    let from_account_id = first_regular_account.id();
    let to_account_id = second_regular_account.id();
    let faucet_account_id = faucet_account_header.id();

    // First Mint necessary token
    let note = mint_note(&mut client, from_account_id, faucet_account_id, NoteType::Private)
        .await
        .1;
    mock_rpc_api.prove_block();
    client.sync_state().await.unwrap();

    //Check that the note is not consumed by the target account
    assert!(matches!(
        client.get_input_note(note.id()).await.unwrap().unwrap().state(),
        InputNoteState::Committed { .. }
    ));

    consume_notes(&mut client, from_account_id, core::slice::from_ref(&note)).await;
    mock_rpc_api.prove_block();
    client.sync_state().await.unwrap();

    assert_account_has_single_asset(&client, from_account_id, faucet_account_id, MINT_AMOUNT).await;

    // Check that the note is consumed by the target account
    let input_note = client.get_input_note(note.id()).await.unwrap().unwrap();
    assert!(matches!(input_note.state(), InputNoteState::ConsumedAuthenticatedLocal { .. }));
    if let InputNoteState::ConsumedAuthenticatedLocal(ConsumedAuthenticatedLocalNoteState {
        submission_data,
        ..
    }) = input_note.state()
    {
        assert_eq!(submission_data.consumer_account, from_account_id);
    } else {
        panic!("Note should be consumed");
    }

    // Do a transfer from first account to second account with Recall. In this situation we'll do
    // the happy path where the `to_account_id` consumes the note
    let from_account_balance = client
        .get_account(from_account_id)
        .await
        .unwrap()
        .unwrap()
        .account()
        .vault()
        .get_balance(faucet_account_id)
        .unwrap_or(0);
    let to_account_balance = client
        .get_account(to_account_id)
        .await
        .unwrap()
        .unwrap()
        .account()
        .vault()
        .get_balance(faucet_account_id)
        .unwrap_or(0);
    let current_block_num = client.get_sync_height().await.unwrap();
    let asset = FungibleAsset::new(faucet_account_id, TRANSFER_AMOUNT).unwrap();
    println!("Running P2IDE tx...");
    let tx_request = TransactionRequestBuilder::new()
        .build_pay_to_id(
            PaymentNoteDescription::new(
                vec![Asset::Fungible(asset)],
                from_account_id,
                to_account_id,
            )
            .with_reclaim_height(current_block_num + RECALL_HEIGHT_DELTA),
            NoteType::Private,
            client.rng(),
        )
        .unwrap();
    Box::pin(client.submit_new_transaction(from_account_id, tx_request.clone()))
        .await
        .unwrap();
    mock_rpc_api.prove_block();
    client.sync_state().await.unwrap();

    // Check that note is committed for the second account to consume
    println!("Fetching Committed Notes...");
    let notes = client.get_input_notes(NoteFilter::Committed).await.unwrap();
    assert!(!notes.is_empty());

    // Make the `to_account_id` consume P2IDE note
    let note_id = tx_request.expected_output_own_notes().pop().unwrap().id();
    println!("Consuming Note...");
    let tx_request = TransactionRequestBuilder::new().build_consume_notes(vec![note_id]).unwrap();
    Box::pin(client.submit_new_transaction(to_account_id, tx_request))
        .await
        .unwrap();
    mock_rpc_api.prove_block();
    client.sync_state().await.unwrap();
    let regular_account = client.get_account(from_account_id).await.unwrap().unwrap();

    // The seed should not be retrieved due to the account not being new
    assert!(!regular_account.account().is_new() && regular_account.seed().is_none());
    assert_eq!(regular_account.account().vault().assets().count(), 1);
    let asset = regular_account.account().vault().assets().next().unwrap();

    // Validate the transferred amounts
    if let Asset::Fungible(fungible_asset) = asset {
        assert_eq!(fungible_asset.amount(), from_account_balance - TRANSFER_AMOUNT);
    } else {
        panic!("Error: Account should have a fungible asset");
    }

    let regular_account: Account = client.get_account(to_account_id).await.unwrap().unwrap().into();
    assert_eq!(regular_account.vault().assets().count(), 1);
    let asset = regular_account.vault().assets().next().unwrap();

    if let Asset::Fungible(fungible_asset) = asset {
        assert_eq!(fungible_asset.amount(), to_account_balance + TRANSFER_AMOUNT);
    } else {
        panic!("Error: Account should have a fungible asset");
    }

    assert_note_cannot_be_consumed_twice(&mut client, to_account_id, note_id).await;
}

#[tokio::test]
async fn p2ide_transfer_consumed_by_sender() {
    let (mut client, mock_rpc_api, authenticator) = Box::pin(create_test_client()).await;

    let (first_regular_account, second_regular_account, faucet_account_header) =
        setup_two_wallets_and_faucet(&mut client, AccountStorageMode::Private, &authenticator)
            .await
            .unwrap();

    let from_account_id = first_regular_account.id();
    let to_account_id = second_regular_account.id();
    let faucet_account_id = faucet_account_header.id();

    // First Mint necessary token
    mint_and_consume(&mut client, from_account_id, faucet_account_id, NoteType::Private).await;
    mock_rpc_api.prove_block();
    client.sync_state().await.unwrap();

    // Do a transfer from first account to second account with Recall. In this situation we'll do
    // the happy path where the `to_account_id` consumes the note
    let from_account_balance = client
        .get_account(from_account_id)
        .await
        .unwrap()
        .unwrap()
        .account()
        .vault()
        .get_balance(faucet_account_id)
        .unwrap_or(0);
    let current_block_num = client.get_sync_height().await.unwrap();
    let asset = FungibleAsset::new(faucet_account_id, TRANSFER_AMOUNT).unwrap();
    println!("Running P2IDE tx...");
    let tx_request = TransactionRequestBuilder::new()
        .build_pay_to_id(
            PaymentNoteDescription::new(
                vec![Asset::Fungible(asset)],
                from_account_id,
                to_account_id,
            )
            .with_reclaim_height(current_block_num + RECALL_HEIGHT_DELTA),
            NoteType::Private,
            client.rng(),
        )
        .unwrap();
    Box::pin(client.submit_new_transaction(from_account_id, tx_request))
        .await
        .unwrap();
    mock_rpc_api.prove_block();
    client.sync_state().await.unwrap();

    // Check that note is committed
    println!("Fetching Committed Notes...");
    let notes = client.get_input_notes(NoteFilter::Committed).await.unwrap();
    assert!(!notes.is_empty());

    // Check that it's still too early to consume
    println!("Consuming Note (too early)...");
    let tx_request = TransactionRequestBuilder::new()
        .build_consume_notes(vec![notes[0].id()])
        .unwrap();
    let transaction_execution_result =
        Box::pin(client.execute_transaction(from_account_id, tx_request)).await;
    assert!(transaction_execution_result.is_err_and(|err| {
        matches!(
            err,
            ClientError::TransactionExecutorError(
                TransactionExecutorError::TransactionProgramExecutionFailed(_)
            )
        )
    }));

    // Wait to consume with the sender account
    println!("Waiting for note to be consumable by sender");
    mock_rpc_api.advance_blocks(RECALL_HEIGHT_DELTA);
    client.sync_state().await.unwrap();

    // Consume the note with the sender account
    println!("Consuming Note...");
    let tx_request = TransactionRequestBuilder::new()
        .build_consume_notes(vec![notes[0].id()])
        .unwrap();
    Box::pin(client.submit_new_transaction(from_account_id, tx_request))
        .await
        .unwrap();
    mock_rpc_api.prove_block();
    client.sync_state().await.unwrap();

    let regular_account = client.get_account(from_account_id).await.unwrap().unwrap();
    // The seed should not be retrieved due to the account not being new
    assert!(!regular_account.account().is_new() && regular_account.seed().is_none());
    assert_eq!(regular_account.account().vault().assets().count(), 1);
    let asset = regular_account.account().vault().assets().next().unwrap();

    // Validate the sender hasn't lost funds
    if let Asset::Fungible(fungible_asset) = asset {
        assert_eq!(fungible_asset.amount(), from_account_balance);
    } else {
        panic!("Error: Account should have a fungible asset");
    }

    let regular_account: Account = client.get_account(to_account_id).await.unwrap().unwrap().into();
    assert_eq!(regular_account.vault().assets().count(), 0);

    // Check that the target can't consume the note anymore
    assert_note_cannot_be_consumed_twice(&mut client, to_account_id, notes[0].id()).await;
}

#[tokio::test]
async fn p2ide_timelocked() {
    let (mut client, mock_rpc_api, authenticator) = Box::pin(create_test_client()).await;

    let (first_regular_account, second_regular_account, faucet_account_header) =
        setup_two_wallets_and_faucet(&mut client, AccountStorageMode::Private, &authenticator)
            .await
            .unwrap();

    let from_account_id = first_regular_account.id();
    let to_account_id = second_regular_account.id();
    let faucet_account_id = faucet_account_header.id();

    // First Mint necessary token
    mint_and_consume(&mut client, from_account_id, faucet_account_id, NoteType::Public).await;
    mock_rpc_api.prove_block();
    client.sync_state().await.unwrap();

    let current_block_num = client.get_sync_height().await.unwrap();

    let asset = FungibleAsset::new(faucet_account_id, TRANSFER_AMOUNT).unwrap();
    let tx_request = TransactionRequestBuilder::new()
        .build_pay_to_id(
            PaymentNoteDescription::new(
                vec![Asset::Fungible(asset)],
                from_account_id,
                to_account_id,
            )
            .with_timelock_height(current_block_num + RECALL_HEIGHT_DELTA)
            .with_reclaim_height(current_block_num),
            NoteType::Public,
            client.rng(),
        )
        .unwrap();
    let note = tx_request.expected_output_own_notes().pop().unwrap();

    Box::pin(client.submit_new_transaction(from_account_id, tx_request))
        .await
        .unwrap();
    mock_rpc_api.prove_block();
    client.sync_state().await.unwrap();

    // Check that it's still too early to consume by both accounts
    let tx_request = TransactionRequestBuilder::new().build_consume_notes(vec![note.id()]).unwrap();
    let results = [
        Box::pin(client.execute_transaction(from_account_id, tx_request.clone())).await,
        Box::pin(client.execute_transaction(to_account_id, tx_request)).await,
    ];
    assert!(results.iter().all(|result| {
        result.as_ref().is_err_and(|err| {
            matches!(
                err,
                ClientError::TransactionExecutorError(
                    TransactionExecutorError::TransactionProgramExecutionFailed(_)
                )
            )
        })
    }));

    // Wait to consume with the target account
    mock_rpc_api.advance_blocks(RECALL_HEIGHT_DELTA);
    client.sync_state().await.unwrap();

    // Consume the note with the target account
    let tx_request = TransactionRequestBuilder::new().build_consume_notes(vec![note.id()]).unwrap();
    Box::pin(client.submit_new_transaction(to_account_id, tx_request))
        .await
        .unwrap();
    mock_rpc_api.prove_block();
    client.sync_state().await.unwrap();

    let target_account: Account = client.get_account(to_account_id).await.unwrap().unwrap().into();
    assert_eq!(target_account.vault().get_balance(faucet_account_id).unwrap(), TRANSFER_AMOUNT);
}

#[tokio::test]
async fn get_consumable_notes() {
    let (mut client, mock_rpc_api, authenticator) = Box::pin(create_test_client()).await;

    let (first_regular_account, second_regular_account, faucet_account_header) =
        setup_two_wallets_and_faucet(&mut client, AccountStorageMode::Private, &authenticator)
            .await
            .unwrap();

    let from_account_id = first_regular_account.id();
    let to_account_id = second_regular_account.id();
    let faucet_account_id = faucet_account_header.id();

    //No consumable notes initially
    assert!(Box::pin(client.get_consumable_notes(None)).await.unwrap().is_empty());

    // First Mint necessary token
    let note = mint_note(&mut client, from_account_id, faucet_account_id, NoteType::Private)
        .await
        .1;
    mock_rpc_api.prove_block();
    client.sync_state().await.unwrap();

    // Check that note is consumable by the account that minted
    assert!(!Box::pin(client.get_consumable_notes(None)).await.unwrap().is_empty());
    assert!(
        !Box::pin(client.get_consumable_notes(Some(from_account_id)))
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        Box::pin(client.get_consumable_notes(Some(to_account_id)))
            .await
            .unwrap()
            .is_empty()
    );

    consume_notes(&mut client, from_account_id, &[note]).await;
    mock_rpc_api.prove_block();
    client.sync_state().await.unwrap();

    //After consuming there are no more consumable notes
    assert!(Box::pin(client.get_consumable_notes(None)).await.unwrap().is_empty());

    // Do a transfer from first account to second account
    let asset = FungibleAsset::new(faucet_account_id, TRANSFER_AMOUNT).unwrap();
    println!("Running P2IDE tx...");
    let tx_request = TransactionRequestBuilder::new()
        .build_pay_to_id(
            PaymentNoteDescription::new(
                vec![Asset::Fungible(asset)],
                from_account_id,
                to_account_id,
            )
            .with_reclaim_height(100.into()),
            NoteType::Private,
            client.rng(),
        )
        .unwrap();
    Box::pin(client.submit_new_transaction(from_account_id, tx_request))
        .await
        .unwrap();
    mock_rpc_api.prove_block();
    client.sync_state().await.unwrap();

    // Check that note is consumable by both accounts
    let consumable_notes = Box::pin(client.get_consumable_notes(None)).await.unwrap();
    let relevant_accounts = &consumable_notes.first().unwrap().1;
    assert_eq!(relevant_accounts.len(), 2);
    assert!(
        !Box::pin(client.get_consumable_notes(Some(from_account_id)))
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        !Box::pin(client.get_consumable_notes(Some(to_account_id)))
            .await
            .unwrap()
            .is_empty()
    );

    // Check that the note is only consumable after block 100 for the account that sent the
    // transaction
    let from_account_relevance = relevant_accounts
        .iter()
        .find(|relevance| relevance.0 == from_account_id)
        .unwrap()
        .1;
    assert_eq!(from_account_relevance, NoteRelevance::After(100));

    // Check that the note is always consumable for the account that received the transaction
    let to_account_relevance = relevant_accounts
        .iter()
        .find(|relevance| relevance.0 == to_account_id)
        .unwrap()
        .1;
    assert_eq!(to_account_relevance, NoteRelevance::Now);
}

#[tokio::test]
async fn get_output_notes() {
    let (mut client, mock_rpc_api, authenticator) = Box::pin(create_test_client()).await;
    let _ = client.sync_state().await.unwrap();
    let (first_regular_account, faucet_account_header) =
        setup_wallet_and_faucet(&mut client, AccountStorageMode::Private, &authenticator)
            .await
            .unwrap();

    let from_account_id = first_regular_account.id();
    let faucet_account_id = faucet_account_header.id();
    let random_account_id = AccountId::try_from(ACCOUNT_ID_REGULAR).unwrap();

    // No output notes initially
    assert!(client.get_output_notes(NoteFilter::All).await.unwrap().is_empty());

    // First Mint necessary token
    let note = mint_note(&mut client, from_account_id, faucet_account_id, NoteType::Private)
        .await
        .1;
    mock_rpc_api.prove_block();
    client.sync_state().await.unwrap();

    // Check that there was an output note but it wasn't consumed
    assert!(client.get_output_notes(NoteFilter::Consumed).await.unwrap().is_empty());
    assert!(!client.get_output_notes(NoteFilter::All).await.unwrap().is_empty());

    consume_notes(&mut client, from_account_id, &[note]).await;
    mock_rpc_api.prove_block();
    client.sync_state().await.unwrap();

    // After consuming, the note is returned when using the [NoteFilter::Consumed] filter
    assert!(!client.get_input_notes(NoteFilter::Consumed).await.unwrap().is_empty());

    // Do a transfer from first account to second account
    let asset = FungibleAsset::new(faucet_account_id, TRANSFER_AMOUNT).unwrap();
    println!("Running P2ID tx...");
    let tx_request = TransactionRequestBuilder::new()
        .build_pay_to_id(
            PaymentNoteDescription::new(
                vec![Asset::Fungible(asset)],
                from_account_id,
                random_account_id,
            ),
            NoteType::Private,
            client.rng(),
        )
        .unwrap();

    let output_note_id = tx_request.expected_output_own_notes().pop().unwrap().id();

    // Before executing, the output note is not found
    assert!(client.get_output_note(output_note_id).await.unwrap().is_none());

    Box::pin(client.submit_new_transaction(from_account_id, tx_request))
        .await
        .unwrap();
    mock_rpc_api.prove_block();
    client.sync_state().await.unwrap();

    // After executing, the note is only found in output notes
    assert!(client.get_output_note(output_note_id).await.unwrap().is_some());
    assert!(client.get_input_note(output_note_id).await.unwrap().is_none());
}

#[tokio::test]
async fn account_rollback() {
    let (builder, mock_rpc_api, authenticator) = Box::pin(create_test_client_builder()).await;

    let mut client = builder.tx_graceful_blocks(Some(TX_GRACEFUL_BLOCKS)).build().await.unwrap();

    client.sync_state().await.unwrap();

    let (regular_account, faucet_account_header) =
        setup_wallet_and_faucet(&mut client, AccountStorageMode::Private, &authenticator)
            .await
            .unwrap();

    let account_id = regular_account.id();
    let faucet_account_id = faucet_account_header.id();

    // Mint a note
    let note = mint_note(&mut client, account_id, faucet_account_id, NoteType::Private).await.1;
    mock_rpc_api.prove_block();
    client.sync_state().await.unwrap();

    consume_notes(&mut client, account_id, &[note]).await;
    mock_rpc_api.prove_block();
    client.sync_state().await.unwrap();

    // Create a transaction but don't submit it to the node
    let asset = FungibleAsset::new(faucet_account_id, TRANSFER_AMOUNT).unwrap();

    let tx_request = TransactionRequestBuilder::new()
        .build_pay_to_id(
            PaymentNoteDescription::new(vec![Asset::Fungible(asset)], account_id, account_id),
            NoteType::Public,
            client.rng(),
        )
        .unwrap();

    // Execute the transaction but don't submit it to the node
    let transaction_result =
        Box::pin(client.execute_transaction(account_id, tx_request)).await.unwrap();
    let tx_id = transaction_result.id();

    // Store the account state before applying the transaction
    let account_before_tx = client.get_account(account_id).await.unwrap().unwrap();
    let account_commitment_before_tx = account_before_tx.account().commitment();

    // Apply the transaction
    let submission_height = client.get_sync_height().await.unwrap();
    let tx_update =
        Box::pin(client.get_transaction_store_update(&transaction_result, submission_height))
            .await
            .unwrap();
    Box::pin(client.apply_transaction_update(tx_update)).await.unwrap();

    // Check that the account state has changed after applying the transaction
    let account_after_tx = client.get_account(account_id).await.unwrap().unwrap();
    let account_commitment_after_tx = account_after_tx.account().commitment();

    assert_ne!(
        account_commitment_before_tx, account_commitment_after_tx,
        "Account commitment should change after applying the transaction"
    );

    // Verify the transaction is in pending state
    let tx_record = client
        .get_transactions(TransactionFilter::All)
        .await
        .unwrap()
        .into_iter()
        .find(|tx| tx.id == tx_id)
        .unwrap();
    assert!(matches!(tx_record.status, TransactionStatus::Pending));

    // Sync the state, which should discard the old pending transaction
    mock_rpc_api.advance_blocks(TX_GRACEFUL_BLOCKS + 1);
    client.sync_state().await.unwrap();

    // Verify the transaction is now discarded
    let tx_record = client
        .get_transactions(TransactionFilter::All)
        .await
        .unwrap()
        .into_iter()
        .find(|tx| tx.id == tx_id)
        .unwrap();

    assert!(matches!(tx_record.status, TransactionStatus::Discarded(DiscardCause::Stale)));

    // Check that the account state has been rolled back after the transaction was discarded
    let account_after_sync = client.get_account(account_id).await.unwrap().unwrap();
    let account_commitment_after_sync = account_after_sync.account().commitment();

    assert_ne!(
        account_commitment_after_sync, account_commitment_after_tx,
        "Account commitment should change after transaction was discarded"
    );
    assert_eq!(
        account_commitment_after_sync, account_commitment_before_tx,
        "Account commitment should be rolled back to the value before the transaction"
    );
}

#[tokio::test]
async fn subsequent_discarded_transactions() {
    let (mut client, mock_rpc_api, keystore) = create_test_client().await;

    let (regular_account, faucet_account_header) =
        setup_wallet_and_faucet(&mut client, AccountStorageMode::Public, &keystore)
            .await
            .unwrap();

    let account_id = regular_account.id();
    let faucet_account_id = faucet_account_header.id();

    let note = mint_note(&mut client, account_id, faucet_account_id, NoteType::Private).await.1;
    mock_rpc_api.prove_block();
    client.sync_state().await.unwrap();

    consume_notes(&mut client, account_id, &[note]).await;
    mock_rpc_api.prove_block();
    client.sync_state().await.unwrap();

    // Create a transaction that will expire in 2 blocks
    let asset = FungibleAsset::new(faucet_account_id, TRANSFER_AMOUNT).unwrap();
    let tx_request = TransactionRequestBuilder::new()
        .expiration_delta(2)
        .build_pay_to_id(
            PaymentNoteDescription::new(vec![Asset::Fungible(asset)], account_id, account_id),
            NoteType::Public,
            client.rng(),
        )
        .unwrap();

    // Execute the transaction but don't submit it to the node
    let transaction_result =
        Box::pin(client.execute_transaction(account_id, tx_request)).await.unwrap();
    let first_tx_id = transaction_result.id();

    let account_before_tx = client.get_account(account_id).await.unwrap().unwrap();

    let submission_height = client.get_sync_height().await.unwrap();
    let tx_update =
        Box::pin(client.get_transaction_store_update(&transaction_result, submission_height))
            .await
            .unwrap();

    Box::pin(client.apply_transaction_update(tx_update)).await.unwrap();

    // Create a second transaction that will not expire
    let asset = FungibleAsset::new(faucet_account_id, TRANSFER_AMOUNT).unwrap();
    let tx_request = TransactionRequestBuilder::new()
        .build_pay_to_id(
            PaymentNoteDescription::new(vec![Asset::Fungible(asset)], account_id, account_id),
            NoteType::Public,
            client.rng(),
        )
        .unwrap();

    // Execute the transaction but don't submit it to the node
    let transaction_result =
        Box::pin(client.execute_transaction(account_id, tx_request)).await.unwrap();
    let second_tx_id = transaction_result.id();
    let submission_height = client.get_sync_height().await.unwrap();
    let tx_update =
        Box::pin(client.get_transaction_store_update(&transaction_result, submission_height))
            .await
            .unwrap();
    Box::pin(client.apply_transaction_update(tx_update)).await.unwrap();

    // Sync the state, which should discard the first transaction
    mock_rpc_api.advance_blocks(3);
    client.sync_state().await.unwrap();

    let account_after_sync = client.get_account(account_id).await.unwrap().unwrap();

    // Verify the first transaction is now discarded
    let first_tx_record = client
        .get_transactions(TransactionFilter::Ids(vec![first_tx_id]))
        .await
        .unwrap()
        .pop()
        .unwrap();

    assert!(matches!(
        first_tx_record.status,
        TransactionStatus::Discarded(DiscardCause::Expired)
    ));

    // Verify the second transaction is also discarded
    let second_tx_record = client
        .get_transactions(TransactionFilter::Ids(vec![second_tx_id]))
        .await
        .unwrap()
        .pop()
        .unwrap();

    println!("Second tx record: {:?}", second_tx_record.status);

    assert!(matches!(
        second_tx_record.status,
        TransactionStatus::Discarded(DiscardCause::DiscardedInitialState)
    ));

    // Check that the account state has been rolled back to the value before both transactions
    assert_eq!(
        account_after_sync.account().commitment(),
        account_before_tx.account().commitment(),
    );
}

#[tokio::test]
async fn missing_recipient_digest() {
    let (mut client, _, keystore) = create_test_client().await;

    let faucet = insert_new_fungible_faucet(&mut client, AccountStorageMode::Private, &keystore)
        .await
        .unwrap();

    let dummy_recipient = NoteRecipient::new(
        Word::default(),
        WellKnownNote::SWAP.script(),
        NoteInputs::new(vec![]).unwrap(),
    );

    let dummy_recipient_digest = dummy_recipient.digest();

    let tx_request = TransactionRequestBuilder::new()
        .expected_output_recipients(vec![dummy_recipient])
        .build_mint_fungible_asset(
            FungibleAsset::new(faucet.id(), 5u64).unwrap(),
            AccountId::try_from(ACCOUNT_ID_PRIVATE_SENDER).unwrap(),
            NoteType::Public,
            client.rng(),
        )
        .unwrap();

    let error = Box::pin(client.submit_new_transaction(faucet.id(), tx_request))
        .await
        .unwrap_err();

    if let ClientError::MissingOutputRecipients(digests) = error {
        assert!(digests == vec![dummy_recipient_digest]);
    }
}

#[tokio::test]
async fn input_note_checks() {
    let (mut client, mock_rpc_api, authenticator) = create_test_client().await;

    let (wallet, faucet) =
        setup_wallet_and_faucet(&mut client, AccountStorageMode::Private, &authenticator)
            .await
            .unwrap();

    let mut mint_notes = vec![];

    for _ in 0..5 {
        mint_notes.push(mint_note(&mut client, wallet.id(), faucet.id(), NoteType::Public).await.1);
        mock_rpc_api.prove_block();
        client.sync_state().await.unwrap();
    }

    let duplicate_note_tx_request = TransactionRequestBuilder::new()
        .build_consume_notes(vec![mint_notes[0].id(), mint_notes[0].id()]);

    assert!(matches!(
        duplicate_note_tx_request,
        Err(TransactionRequestError::DuplicateInputNote(note_id)) if note_id == mint_notes[0].id()
    ));

    let tx_request = TransactionRequestBuilder::new()
        .build_consume_notes(mint_notes.iter().map(Note::id).collect())
        .unwrap();

    let transaction_result =
        Box::pin(client.execute_transaction(wallet.id(), tx_request)).await.unwrap();
    let transaction = transaction_result.executed_transaction().clone();

    let input_notes = transaction.input_notes().iter();

    // Check that the input notes have the same order as the original notes
    for (i, input_note) in input_notes.enumerate() {
        assert_eq!(input_note.id(), mint_notes[i].id());
    }

    let proven_transaction = client.prove_transaction(&transaction_result).await.unwrap();
    let submission_height = client
        .submit_proven_transaction(proven_transaction, &transaction_result)
        .await
        .unwrap();
    let tx_update =
        Box::pin(client.get_transaction_store_update(&transaction_result, submission_height))
            .await
            .unwrap();
    Box::pin(client.apply_transaction_update(tx_update)).await.unwrap();
    mock_rpc_api.prove_block();
    client.sync_state().await.unwrap();

    // Check that using consumed notes will return an error
    let consumed_note_tx_request = TransactionRequestBuilder::new()
        .build_consume_notes(vec![mint_notes[0].id()])
        .unwrap();
    let error = Box::pin(client.submit_new_transaction(wallet.id(), consumed_note_tx_request))
        .await
        .unwrap_err();

    assert!(matches!(
        error,
        ClientError::TransactionRequestError(TransactionRequestError::InputNoteAlreadyConsumed(_))
    ));

    // Check that adding an authenticated note that is not tracked by the client will return an
    // error
    let missing_authenticated_note_tx_request = TransactionRequestBuilder::new()
        .build_consume_notes(vec![EMPTY_WORD.into()])
        .unwrap();
    let error =
        Box::pin(client.submit_new_transaction(wallet.id(), missing_authenticated_note_tx_request))
            .await
            .unwrap_err();

    assert!(matches!(
        error,
        ClientError::TransactionRequestError(
            TransactionRequestError::MissingAuthenticatedInputNote(_)
        )
    ));
}

#[tokio::test]
async fn swap_chain_test() {
    // This test simulates a "swap chain" scenario with multiple wallets and fungible assets.
    // 1. It creates a number wallet/faucet pairs, each wallet holding an asset minted by its paired
    //    faucet.
    // 2. For each consecutive pair, it creates a swap transaction where wallet N offers its asset
    //    and requests the asset of wallet N+1.
    // 3. The last wallet, which didn't generate any swaps, holds the asset that the wallet N-1
    //    requested, which in turn was the asset requested by wallet N-2, and so on.
    // 4. The test then consumes all swap notes (in reverse order) in a single transaction against
    //    the last wallet.
    // 5. Although the last wallet doesn't contain any of the intermediate requested assets, it
    //    should be able to consume the swap notes because it will hold the requested asset for each
    //    step and gain the needed asset for the next. This can only happen if the notes are
    //    consumed in the specified order.
    // 6. Finally, it asserts that the last wallet now owns the asset originally held by the first
    //    wallet, verifying that the whole swap chain was successful.

    let (mut client, mock_rpc_api, keystore) = create_test_client().await;

    // Generate a few account pairs with a fungible asset that can be used for swaps.
    let mut account_pairs = vec![];
    for _ in 0..3 {
        let (wallet, faucet) =
            setup_wallet_and_faucet(&mut client, AccountStorageMode::Private, &keystore)
                .await
                .unwrap();
        mint_and_consume(&mut client, wallet.id(), faucet.id(), NoteType::Private).await;
        mock_rpc_api.prove_block();
        client.sync_state().await.unwrap();

        account_pairs.push((wallet, faucet));
    }

    // Generate swap notes.
    // Except for the last, each wallet N will offer it's faucet N asset and request a faucet N+1
    // asset.
    let mut swap_notes = vec![];
    for pairs in account_pairs.windows(2) {
        let tx_request = TransactionRequestBuilder::new()
            .build_swap(
                &SwapTransactionData::new(
                    pairs[0].0.id(),
                    Asset::Fungible(FungibleAsset::new(pairs[0].1.id(), 1).unwrap()),
                    Asset::Fungible(FungibleAsset::new(pairs[1].1.id(), 1).unwrap()),
                ),
                NoteType::Private,
                NoteType::Private,
                client.rng(),
            )
            .unwrap();

        // The notes are inserted in reverse order because the first note to be consumed will be the
        // last one generated.
        swap_notes.insert(0, tx_request.expected_output_own_notes()[0].id());
        Box::pin(client.submit_new_transaction(pairs[0].0.id(), tx_request))
            .await
            .unwrap();
        mock_rpc_api.prove_block();
        client.sync_state().await.unwrap();
    }

    // The last wallet didn't generate any swap notes and has the asset needed to start the swap
    // chain.
    let last_wallet = account_pairs.last().unwrap().0.id();

    // Trying to consume the notes in another order will fail.
    let tx_request = TransactionRequestBuilder::new()
        .build_consume_notes(swap_notes.iter().rev().copied().collect())
        .unwrap();
    let error = Box::pin(client.submit_new_transaction(last_wallet, tx_request))
        .await
        .unwrap_err();
    assert!(matches!(
        error,
        ClientError::TransactionExecutorError(
            TransactionExecutorError::TransactionProgramExecutionFailed(_)
        )
    ));

    let tx_request = TransactionRequestBuilder::new().build_consume_notes(swap_notes).unwrap();
    Box::pin(client.submit_new_transaction(last_wallet, tx_request)).await.unwrap();

    // At the end, the last wallet should have the asset of the first wallet.
    let last_wallet_account = client.get_account(last_wallet).await.unwrap().unwrap();
    assert_eq!(
        last_wallet_account
            .account()
            .vault()
            .get_balance(account_pairs[0].1.id())
            .unwrap(),
        1
    );
}

#[tokio::test]
async fn empty_storage_map() {
    let (mut client, _, keystore) = create_test_client().await;

    let storage_map = StorageMap::new();

    let component = AccountComponent::compile(
        "export.dummy
                nop
            end"
        .to_string(),
        TransactionKernel::assembler(),
        vec![StorageSlot::Map(storage_map)],
    )
    .unwrap()
    .with_supports_all_types();

    let key_pair = SecretKey::new();
    let pub_key = key_pair.public_key();

    keystore.add_key(&AuthSecretKey::RpoFalcon512(key_pair.clone())).unwrap();

    let mut init_seed = [0u8; 32];
    client.rng().fill_bytes(&mut init_seed);

    let account = AccountBuilder::new(init_seed)
        .account_type(AccountType::RegularAccountImmutableCode)
        .storage_mode(AccountStorageMode::Public)
        .with_auth_component(AuthRpoFalcon512::new(pub_key.to_commitment().into()))
        .with_component(BasicWallet)
        .with_component(component)
        .build()
        .unwrap();

    let account_id = account.id();

    client.add_account(&account, false).await.unwrap();

    let fetched_account = client.get_account(account_id).await.unwrap().unwrap();

    assert_eq!(account.storage(), fetched_account.account().storage());
}

const MAP_KEY: [Felt; 4] = [Felt::new(42), Felt::new(42), Felt::new(42), Felt::new(42)];
const BUMP_MAP_CODE: &str = "export.bump_map_item
                    # map key
                    push.{map_key}
                    # item index
                    push.0
                    # => [index, KEY]
                    exec.::miden::account::get_map_item
                    add.1
                    push.{map_key}
                    push.0
                    # => [index, KEY, BUMPED_VALUE]
                    exec.::miden::account::set_map_item
                    dropw
                    # => [OLD_VALUE]
                    dupw
                    push.0
                    # Set a new item each time as the value keeps changing
                    exec.::miden::account::set_map_item
                    dropw dropw
                end";

#[tokio::test]
async fn storage_and_vault_proofs() {
    let (mut client, mock_rpc_api, keystore) = create_test_client().await;

    // Create an account that will accept assets (basic wallet) but also that has a storage map that
    // can be updated.
    let mut storage_map = StorageMap::new();
    storage_map
        .insert(MAP_KEY.into(), [Felt::new(0), Felt::new(0), Felt::new(0), Felt::new(1)].into())
        .unwrap();

    let bump_item_component = AccountComponent::compile(
        BUMP_MAP_CODE.replace("{map_key}", &Word::from(MAP_KEY).to_hex()),
        TransactionKernel::assembler(),
        vec![StorageSlot::Map(storage_map)],
    )
    .unwrap()
    .with_supports_all_types();

    // Build script that bumps the storage map item and adds a new one each time.
    let assembler: Assembler = TransactionKernel::assembler().with_debug_mode(true);
    let source_manager = Arc::new(DefaultSourceManager::default());
    let module = Module::parser(ModuleKind::Library)
        .parse_str(
            LibraryPath::new("external_contract::bump_item_contract").unwrap(),
            BUMP_MAP_CODE.replace("{map_key}", &Word::from(MAP_KEY).to_hex()),
            &source_manager,
        )
        .unwrap();
    let library = assembler.clone().assemble_library([module]).unwrap();
    let tx_script = ScriptBuilder::new(true)
        .with_dynamically_linked_library(&library)
        .unwrap()
        .compile_tx_script(
            "use.external_contract::bump_item_contract
            begin
                call.bump_item_contract::bump_map_item
            end",
        )
        .unwrap();

    let key_pair = SecretKey::new();
    let pub_key = key_pair.public_key();

    keystore.add_key(&AuthSecretKey::RpoFalcon512(key_pair.clone())).unwrap();

    let mut init_seed = [0u8; 32];
    client.rng().fill_bytes(&mut init_seed);

    let account = AccountBuilder::new(init_seed)
        .account_type(AccountType::RegularAccountImmutableCode)
        .storage_mode(AccountStorageMode::Public)
        .with_auth_component(AuthRpoFalcon512::new(pub_key.to_commitment().into()))
        .with_component(BasicWallet)
        .with_component(bump_item_component)
        .build()
        .unwrap();

    client.add_account(&account, false).await.unwrap();

    let account_id = account.id();

    // Add assets and modify storage map multiple times
    for _ in 0..5 {
        let faucet_account =
            insert_new_fungible_faucet(&mut client, AccountStorageMode::Public, &keystore)
                .await
                .unwrap();

        let faucet_account_id = faucet_account.id();

        mint_and_consume(&mut client, account_id, faucet_account_id, NoteType::Private).await;
        mock_rpc_api.prove_block();
        client.sync_state().await.unwrap();

        let tx_request = TransactionRequestBuilder::new()
            .custom_script(tx_script.clone())
            .build()
            .unwrap();
        Box::pin(client.submit_new_transaction(account_id, tx_request)).await.unwrap();
        mock_rpc_api.prove_block();
        client.sync_state().await.unwrap();

        // Check that retrieved vault and storage match with the account.
        let account: Account = client.get_account(account_id).await.unwrap().unwrap().into();

        let storage = client.test_store().get_account_storage(account_id).await.unwrap();
        let vault = client.test_store().get_account_vault(account_id).await.unwrap();

        assert_eq!(account.storage().commitment(), storage.commitment());
        assert_eq!(account.vault().root(), vault.root());

        // Check that specific asset proof matches the one in the vault
        let (asset, witness) = client
            .test_store()
            .get_account_asset(account_id, faucet_account_id.prefix())
            .await
            .unwrap()
            .unwrap();

        let expected_witness = AssetWitness::new(vault.open(asset.vault_key()).into()).unwrap();
        assert_eq!(witness, expected_witness);

        // Check that specific map item proof matches the one in the storage
        let (value, proof) = client
            .test_store()
            .get_account_map_item(account_id, 1, MAP_KEY.into())
            .await
            .unwrap();

        let StorageSlot::Map(map) = storage.slots().get(1).unwrap() else {
            panic!("Expected a map storage slot");
        };

        assert_eq!(value, map.get(&MAP_KEY.into()));
        assert_eq!(proof, map.open(&MAP_KEY.into()));
    }
}

#[tokio::test]
async fn account_addresses_basic_wallet() {
    // generate test client with a random store name
    let (mut client, _rpc_api, _) = Box::pin(create_test_client()).await;

    let account = Account::mock(
        ACCOUNT_ID_PUBLIC_FUNGIBLE_FAUCET_2,
        AuthRpoFalcon512::new(PublicKeyCommitment::from(EMPTY_WORD)),
    );

    client.add_account(&account, false).await.unwrap();
    let retrieved_acc = client.get_account(account.id()).await.unwrap().unwrap();

    let unspecified_default_address =
        Address::AccountId(AccountIdAddress::new(account.id(), AddressInterface::Unspecified));
    assert!(retrieved_acc.addresses().contains(&unspecified_default_address));

    // Even when the account has a basic wallet, the address list should not contain it by default
    let basic_wallet_address =
        Address::AccountId(AccountIdAddress::new(account.id(), AddressInterface::BasicWallet));
    assert!(!retrieved_acc.addresses().contains(&basic_wallet_address));
}

#[tokio::test]
async fn account_addresses_non_basic_wallet() {
    // generate test client with a random store name
    let (mut client, _rpc_api, _) = Box::pin(create_test_client()).await;

    let account = Account::mock_non_fungible_faucet(ACCOUNT_ID_PUBLIC_NON_FUNGIBLE_FAUCET);

    client.add_account(&account, false).await.unwrap();
    let retrieved_acc = client.get_account(account.id()).await.unwrap().unwrap();

    let unspecified_default_address =
        Address::AccountId(AccountIdAddress::new(account.id(), AddressInterface::Unspecified));
    assert!(retrieved_acc.addresses().contains(&unspecified_default_address));

    let basic_wallet_address =
        Address::AccountId(AccountIdAddress::new(account.id(), AddressInterface::BasicWallet));
    assert!(!retrieved_acc.addresses().contains(&basic_wallet_address));
}

// HELPERS
// ================================================================================================

pub async fn create_test_client()
-> (MockClient<FilesystemKeyStore<StdRng>>, MockRpcApi, FilesystemKeyStore<StdRng>) {
    let (builder, rpc_api, keystore) = Box::pin(create_test_client_builder()).await;
    let mut client = builder.build().await.unwrap();
    client.ensure_genesis_in_place().await.unwrap();

    (client, rpc_api, keystore)
}

pub async fn create_test_client_builder()
-> (ClientBuilder<TestClientKeyStore>, MockRpcApi, FilesystemKeyStore<StdRng>) {
    let mut rng = rand::rng();
    let coin_seed: [u64; 4] = rng.random();

    let rng = RpoRandomCoin::new(coin_seed.map(Felt::new).into());

    let keystore_path = temp_dir();
    let keystore = FilesystemKeyStore::new(keystore_path.clone()).unwrap();

    let rpc_api = MockRpcApi::new(Box::pin(create_prebuilt_mock_chain()).await);
    let arc_rpc_api = Arc::new(rpc_api.clone());

    let builder = ClientBuilder::new()
        .rpc(arc_rpc_api)
        .rng(Box::new(rng))
        .sqlite_store(create_test_store_path())
        .filesystem_keystore(keystore_path.to_str().unwrap())
        .in_debug_mode(DebugMode::Enabled)
        .tx_graceful_blocks(None);

    (builder, rpc_api, keystore)
}

pub async fn create_prebuilt_mock_chain() -> MockChain {
    let mut mock_chain_builder = MockChainBuilder::new();
    let mock_account = mock_chain_builder
        .add_existing_mock_account(miden_testing::Auth::IncrNonce)
        .unwrap();

    let note_first =
        NoteBuilder::new(mock_account.id(), RpoRandomCoin::new([0, 0, 0, 0].map(Felt::new).into()))
            .note_type(NoteType::Public)
            .tag(NoteTag::for_public_use_case(0, 0, NoteExecutionMode::Local).unwrap().into())
            .build()
            .unwrap();

    let note_second =
        NoteBuilder::new(mock_account.id(), RpoRandomCoin::new([0, 0, 0, 1].map(Felt::new).into()))
            .note_type(NoteType::Public)
            .tag(NoteTag::for_local_use_case(0, 0).unwrap().into())
            .build()
            .unwrap();
    let spawn_note_1 =
        mock_chain_builder.add_spawn_note(std::slice::from_ref(&note_first)).unwrap();
    let spawn_note_2 =
        mock_chain_builder.add_spawn_note(std::slice::from_ref(&note_second)).unwrap();
    let mut mock_chain = mock_chain_builder.build().unwrap();

    // Block 1: Create first note
    let tx = Box::pin(
        mock_chain
            .build_tx_context(TxContextInput::AccountId(mock_account.id()), &[], &[spawn_note_1])
            .unwrap()
            .extend_expected_output_notes(vec![OutputNote::Full(note_first)])
            .build()
            .unwrap()
            .execute(),
    )
    .await
    .unwrap();
    mock_chain.add_pending_executed_transaction(&tx).unwrap();
    mock_chain.prove_next_block().unwrap();

    // Block 2
    mock_chain.prove_next_block().unwrap();

    // Block 3
    mock_chain.prove_next_block().unwrap();

    // Block 4: Create second note

    let tx = Box::pin(
        mock_chain
            .build_tx_context(mock_account.id(), &[], &[spawn_note_2])
            .unwrap()
            .extend_expected_output_notes(vec![OutputNote::Full(note_second.clone())])
            .build()
            .unwrap()
            .execute(),
    )
    .await
    .unwrap();
    mock_chain.add_pending_executed_transaction(&tx).unwrap();

    mock_chain.prove_next_block().unwrap();

    let transaction = Box::pin(
        mock_chain
            .build_tx_context(mock_account.id(), &[], &[note_second])
            .unwrap()
            .build()
            .unwrap()
            .execute(),
    )
    .await
    .unwrap();

    // Block 5: Consume (nullify) second note
    mock_chain.add_pending_executed_transaction(&transaction).unwrap();
    mock_chain.prove_next_block().unwrap();

    mock_chain
}

async fn insert_new_wallet(
    client: &mut TestClient,
    storage_mode: AccountStorageMode,
    keystore: &FilesystemKeyStore<StdRng>,
) -> Result<Account, ClientError> {
    let key_pair = SecretKey::with_rng(&mut client.rng());
    let pub_key = key_pair.public_key();

    keystore.add_key(&AuthSecretKey::RpoFalcon512(key_pair)).unwrap();

    let mut init_seed = [0u8; 32];
    client.rng().fill_bytes(&mut init_seed);

    let account = AccountBuilder::new(init_seed)
        .account_type(AccountType::RegularAccountImmutableCode)
        .storage_mode(storage_mode)
        .with_auth_component(AuthRpoFalcon512::new(pub_key.to_commitment().into()))
        .with_component(BasicWallet)
        .build()
        .unwrap();

    client.add_account(&account, false).await?;

    Ok(account)
}

async fn insert_new_fungible_faucet(
    client: &mut TestClient,
    storage_mode: AccountStorageMode,
    keystore: &FilesystemKeyStore<StdRng>,
) -> Result<Account, ClientError> {
    let key_pair = SecretKey::with_rng(client.rng());
    let pub_key = key_pair.public_key();

    keystore.add_key(&AuthSecretKey::RpoFalcon512(key_pair)).unwrap();

    // we need to use an initial seed to create the wallet account
    let mut init_seed = [0u8; 32];
    client.rng().fill_bytes(&mut init_seed);

    let symbol = TokenSymbol::new("TEST").unwrap();
    let max_supply = Felt::try_from(9_999_999_u64.to_le_bytes().as_slice())
        .expect("u64 can be safely converted to a field element");

    let account = AccountBuilder::new(init_seed)
        .account_type(AccountType::FungibleFaucet)
        .storage_mode(storage_mode)
        .with_auth_component(AuthRpoFalcon512::new(pub_key.to_commitment().into()))
        .with_component(BasicFungibleFaucet::new(symbol, 10, max_supply).unwrap())
        .build()
        .unwrap();

    client.add_account(&account, false).await?;
    Ok(account)
}
