use anyhow::Result;
use miden_client::account::component::{
    AuthFalcon512RpoAcl,
    AuthFalcon512RpoAclConfig,
    BasicWallet,
};
use miden_client::account::{Account, AccountBuilder, AccountId, AccountStorageMode, AccountType};
use miden_client::assembly::CodeBuilder;
use miden_client::asset::{Asset, FungibleAsset};
use miden_client::auth::{AuthSchemeId, TransactionAuthenticator};
use miden_client::crypto::FeltRng;
use miden_client::crypto::rpo_falcon512::SecretKey;
use miden_client::note::{
    Note,
    NoteAssets,
    NoteDetails,
    NoteFile,
    NoteInputs,
    NoteMetadata,
    NoteRecipient,
    NoteScript,
    NoteTag,
    NoteType,
    build_p2id_recipient,
};
use miden_client::store::{InputNoteState, TransactionFilter};
use miden_client::testing::common::*;
use miden_client::transaction::{OutputNote, TransactionRequestBuilder};
use miden_client::{Client, ClientRng, Word};
use rand::RngCore;

use crate::tests::config::ClientConfig;

// PASS-THROUGH TRANSACTIONS (change sender from Alice -> Pass-through account)
// ================================================================================================

pub async fn test_pass_through(client_config: ClientConfig) -> Result<()> {
    const ASSET_AMOUNT: u64 = 1;
    let (mut client, authenticator_1) = client_config.clone().into_client().await?;

    // Workaround to show that importing the note into another client works
    let mut client_config_2 = client_config.as_parts();
    client_config_2.2 = create_test_store_path();
    let client_config_2 = ClientConfig {
        rpc_endpoint: client_config_2.0,
        rpc_timeout_ms: client_config_2.1,
        store_config: client_config_2.2,
        auth_path: client_config_2.3,
    };
    let (mut client_2, authenticator_2) = client_config_2.into_client().await?;

    wait_for_node(&mut client).await;
    client.sync_state().await?;
    client_2.sync_state().await?;

    // Create Client basic wallet (We'll call it accountA)
    let (sender, ..) = insert_new_wallet(
        &mut client,
        AccountStorageMode::Private,
        &authenticator_1,
        AuthSchemeId::Falcon512Rpo,
    )
    .await?;
    let (target, ..) = insert_new_wallet(
        &mut client_2,
        AccountStorageMode::Private,
        &authenticator_2,
        AuthSchemeId::Falcon512Rpo,
    )
    .await?;

    let pass_through_account = create_pass_through_account(&mut client).await?;

    // Create client with faucets BTC faucet
    let (btc_faucet_account, ..) = insert_new_fungible_faucet(
        &mut client,
        AccountStorageMode::Private,
        &authenticator_1,
        AuthSchemeId::Falcon512Rpo,
    )
    .await?;

    // mint 1000 BTC for accountA
    println!("minting 1000 btc for account A");

    let tx_id =
        mint_and_consume(&mut client, sender.id(), btc_faucet_account.id(), NoteType::Public).await;
    wait_for_tx(&mut client, tx_id).await?;

    // Create a note that we will send to a pass-through account
    println!("creating note with accountA");
    let asset = FungibleAsset::new(btc_faucet_account.id(), ASSET_AMOUNT)?;

    let (pass_through_note_1, pass_through_note_details_1) =
        create_pass_through_note(sender.id(), target.id(), asset.into(), client.rng())?;

    let (pass_through_note_2, pass_through_note_details_2) =
        create_pass_through_note(sender.id(), target.id(), asset.into(), client.rng())?;

    let tx_request = TransactionRequestBuilder::new()
        .own_output_notes(vec![
            OutputNote::Full(pass_through_note_1.clone()),
            OutputNote::Full(pass_through_note_2.clone()),
        ])
        .build()?;

    execute_tx_and_sync(&mut client, sender.id(), tx_request).await?;

    println!("consuming pass-through note");

    client
        .import_notes(&[
            NoteFile::NoteId(pass_through_note_1.id()),
            NoteFile::NoteId(pass_through_note_2.id()),
        ])
        .await?;
    client.sync_state().await?;
    let input_note_record = client.get_input_note(pass_through_note_1.id()).await?.unwrap();
    assert!(matches!(input_note_record.state(), InputNoteState::Committed { .. }));
    let input_note_record = client.get_input_note(pass_through_note_2.id()).await?.unwrap();
    assert!(matches!(input_note_record.state(), InputNoteState::Committed { .. }));

    let tx_request = TransactionRequestBuilder::new()
        .expected_output_recipients(vec![pass_through_note_details_1.recipient().clone()])
        .build_consume_notes(vec![pass_through_note_1])
        .unwrap();

    let tx_id = client
        .submit_new_transaction(pass_through_account.id(), tx_request.clone())
        .await?;

    wait_for_tx(&mut client, tx_id).await?;

    let tx_record = client
        .get_transactions(TransactionFilter::Ids(vec![tx_id]))
        .await?
        .pop()
        .unwrap();

    assert_eq!(
        tx_record.details.output_notes.get_note(0).metadata().sender(),
        pass_through_account.id()
    );

    let pass_through_before_second_tx = client
        .get_account(pass_through_account.id())
        .await?
        .expect("pass-through account should exist");

    // Storing commitment to check later that (final_acc.commitment == initial_acc.commitment)
    let commitment_before_second_tx = pass_through_before_second_tx.account_data().commitment();

    // now try another transaction against the pass-through account
    let tx_request = TransactionRequestBuilder::new()
        .expected_output_recipients(vec![pass_through_note_details_2.recipient().clone()])
        .build_consume_notes(vec![pass_through_note_2])
        .unwrap();

    let tx_id = client
        .submit_new_transaction(pass_through_account.id(), tx_request.clone())
        .await?;

    wait_for_tx(&mut client, tx_id).await?;

    let tx_record = client
        .get_transactions(TransactionFilter::Ids(vec![tx_id]))
        .await?
        .pop()
        .unwrap();

    assert_eq!(
        tx_record.details.output_notes.get_note(0).metadata().sender(),
        pass_through_account.id()
    );

    let pass_through_after_second_tx = client
        .get_account(pass_through_account.id())
        .await?
        .expect("pass-through account should exist");

    assert_eq!(
        pass_through_after_second_tx.account_data().commitment(),
        commitment_before_second_tx,
        "pass-through transaction should not change account commitment"
    );

    Ok(())
}

// HELPERS
// ================================================================================================

async fn create_pass_through_account<AUTH: TransactionAuthenticator>(
    client: &mut Client<AUTH>,
) -> Result<Account> {
    let mut init_seed = [0u8; 32];
    client.rng().fill_bytes(&mut init_seed);

    let key_pair = SecretKey::with_rng(client.rng());
    let pub_key = key_pair.public_key().to_commitment();

    let acl_config = AuthFalcon512RpoAclConfig::new()
        .with_allow_unauthorized_input_notes(true)
        .with_allow_unauthorized_output_notes(true);

    let auth_component = AuthFalcon512RpoAcl::new(pub_key.into(), acl_config).unwrap();

    let account = AccountBuilder::new(init_seed)
        .account_type(AccountType::RegularAccountImmutableCode)
        .storage_mode(AccountStorageMode::Private)
        .with_auth_component(auth_component)
        .with_component(BasicWallet)
        .build()
        .unwrap();

    client.add_account(&account, false).await?;
    Ok(account)
}

fn get_pass_through_note_script() -> NoteScript {
    let note_script_code = include_str!("../asm/PASS_THROUGH.masm");

    CodeBuilder::new().compile_note_script(note_script_code).unwrap()
}

// Creates a note eventually meant for the target account.
// First, the note is processed by the pass-through account.
// The output note script guarantees the output of the processing is `target`.
fn create_pass_through_note(
    sender: AccountId,
    target: AccountId,
    asset: Asset,
    rng: &mut ClientRng,
) -> Result<(Note, NoteDetails)> {
    let note_script = get_pass_through_note_script();

    let asset_word: Word = asset.into();

    let target_recipient = build_p2id_recipient(target, rng.draw_word())?;

    let inputs = NoteInputs::new(vec![
        asset_word[0],
        asset_word[1],
        asset_word[2],
        asset_word[3],
        target_recipient.digest()[0],
        target_recipient.digest()[1],
        target_recipient.digest()[2],
        target_recipient.digest()[3],
        NoteType::Public.into(),
        NoteTag::with_account_target(target).into(),
    ])?;

    let serial_num = rng.draw_word();
    let pass_through_recipient = NoteRecipient::new(serial_num, note_script, inputs);

    let metadata =
        NoteMetadata::new(sender, NoteType::Public, NoteTag::with_account_target(target));
    let note = Note::new(NoteAssets::new(vec![asset])?, metadata, pass_through_recipient);

    let pass_through_note_details =
        NoteDetails::new(NoteAssets::new(vec![asset])?, target_recipient);
    Ok((note, pass_through_note_details))
}
