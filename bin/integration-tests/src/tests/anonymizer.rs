use anyhow::Result;
use miden_client::account::component::{
    AuthRpoFalcon512Acl,
    AuthRpoFalcon512AclConfig,
    BasicWallet,
};
use miden_client::account::{Account, AccountBuilder, AccountId, AccountStorageMode, AccountType};
use miden_client::asset::{Asset, FungibleAsset};
use miden_client::auth::TransactionAuthenticator;
use miden_client::crypto::{FeltRng, SecretKey};
use miden_client::note::{
    Note,
    NoteAssets,
    NoteDetails,
    NoteExecutionHint,
    NoteFile,
    NoteInputs,
    NoteMetadata,
    NoteRecipient,
    NoteScript,
    NoteTag,
    NoteType,
    build_p2id_recipient,
};
use miden_client::store::InputNoteState;
use miden_client::testing::common::*;
use miden_client::transaction::{OutputNote, TransactionRequestBuilder};
use miden_client::{Client, ClientRng, Felt, ScriptBuilder, Word};
use rand::RngCore;

use crate::tests::config::ClientConfig;

// SWAP FULLY ONCHAIN
// ================================================================================================

pub async fn test_anonymizer(client_config: ClientConfig) -> Result<()> {
    const ASSET_AMOUNT: u64 = 1;
    let (mut client, authenticator_1) = client_config.clone().into_client().await?;

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
    let (sender, ..) =
        insert_new_wallet(&mut client, AccountStorageMode::Private, &authenticator_1).await?;
    let (target, ..) =
        insert_new_wallet(&mut client_2, AccountStorageMode::Private, &authenticator_2).await?;

    let anonymizer_account = create_anonymizer_account(&mut client).await?;

    // Create client with faucets BTC faucet
    let (btc_faucet_account, ..) =
        insert_new_fungible_faucet(&mut client, AccountStorageMode::Private, &authenticator_1)
            .await?;

    // mint 1000 BTC for accountA
    println!("minting 1000 btc for account A");

    let tx_id =
        mint_and_consume(&mut client, sender.id(), btc_faucet_account.id(), NoteType::Public).await;
    wait_for_tx(&mut client, tx_id).await?;

    // Create a note that we will send to an anonymizer account
    println!("creating note with accountA");
    let asset = FungibleAsset::new(btc_faucet_account.id(), ASSET_AMOUNT)?;

    let (anonymizer_note, anonymized_note_details) =
        create_anonymizer_note(sender.id(), target.id(), asset.into(), client.rng())?;

    let tx_request = TransactionRequestBuilder::new()
        .own_output_notes(vec![OutputNote::Full(anonymizer_note.clone())])
        .expected_future_notes(vec![(
            anonymized_note_details,
            NoteTag::from_account_id(target.id()),
        )])
        .build()?;

    execute_tx_and_sync(&mut client, sender.id(), tx_request).await?;

    println!("consuming anonymizer note");

    client.import_note(NoteFile::NoteId(anonymizer_note.id())).await?;
    client.sync_state().await?;

    client_2.import_note(NoteFile::NoteId(anonymizer_note.id())).await?;
    client_2.sync_state().await?;
    let input_note_record = client_2.get_input_note(anonymizer_note.id()).await?.unwrap();
    assert!(matches!(input_note_record.state(), InputNoteState::Committed { .. }));

    let input_note_record = client.get_input_note(anonymizer_note.id()).await?.unwrap();
    // state is unverified :(
    assert!(matches!(input_note_record.state(), InputNoteState::Committed { .. }));

    let tx_request = TransactionRequestBuilder::new()
        .build_consume_notes(vec![anonymizer_note.id()])
        .unwrap();
    execute_tx_and_sync(&mut client, anonymizer_account.id(), tx_request).await?;

    Ok(())
}

// HELPERS
// ================================================================================================
async fn create_anonymizer_account<AUTH: TransactionAuthenticator>(
    client: &mut Client<AUTH>,
) -> Result<Account> {
    let mut init_seed = [0u8; 32];
    client.rng().fill_bytes(&mut init_seed);

    let key_pair = SecretKey::with_rng(client.rng());
    let pub_key = key_pair.public_key();

    let acl_config = AuthRpoFalcon512AclConfig::new()
        .with_allow_unauthorized_input_notes(true)
        .with_allow_unauthorized_output_notes(true);

    let auth_component = AuthRpoFalcon512Acl::new(pub_key, acl_config).unwrap();

    let (account, seed) = AccountBuilder::new(init_seed)
        .account_type(AccountType::RegularAccountImmutableCode)
        .storage_mode(AccountStorageMode::Private)
        .with_auth_component(auth_component)
        .with_component(BasicWallet)
        .build()
        .unwrap();

    client.add_account(&account, Some(seed), false).await?;
    Ok(account)
}

fn get_anonymizer_note_script() -> NoteScript {
    let note_script_code = include_str!("../asm/ANONYMIZER.masm");

    ScriptBuilder::new(true).compile_note_script(note_script_code).unwrap()
}

// Creates a note eventually meant for the target account.
// First, the note is mixed by the anonymizer account.
// The output note script guarantees the output of the mixing is `target`.
fn create_anonymizer_note(
    sender: AccountId,
    target: AccountId,
    asset: Asset,
    rng: &mut ClientRng,
) -> Result<(Note, NoteDetails)> {
    let note_script = get_anonymizer_note_script();

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
        NoteExecutionHint::always().into(),
        NoteType::Public.into(),
        Felt::new(0u64),
        NoteTag::from_account_id(target).into(),
    ])?;

    let serial_num = rng.draw_word();
    let anonymizer_recipient = NoteRecipient::new(serial_num, note_script, inputs);

    let metadata = NoteMetadata::new(
        sender,
        NoteType::Public,
        NoteTag::from_account_id(sender), // this needs to change
        NoteExecutionHint::always(),
        Felt::new(0u64),
    )?;
    let note = Note::new(NoteAssets::new(vec![asset])?, metadata, anonymizer_recipient);

    let anonymized_note_details = NoteDetails::new(NoteAssets::new(vec![asset])?, target_recipient);
    Ok((note, anonymized_note_details))
}
