use std::vec;

use anyhow::Result;
use miden_client::account::{AccountId, AccountStorageMode};
use miden_client::asset::{Asset, FungibleAsset};
use miden_client::auth::RPO_FALCON_SCHEME_ID;
use miden_client::crypto::FeltRng;
use miden_client::note::{
    NetworkAccountTarget, Note, NoteAssets, NoteAttachment, NoteDetails, NoteExecutionHint,
    NoteFile, NoteMetadata, NoteRecipient, NoteStorage, NoteTag, NoteType, P2idNote,
};
use miden_client::store::{InputNoteState, TransactionFilter};
use miden_client::testing::common::{
    execute_tx_and_sync, insert_new_fungible_faucet, insert_new_wallet, mint_and_consume,
    wait_for_tx,
};
use miden_client::transaction::{OutputNote, TransactionRequestBuilder};
use miden_client::{ClientRng, Word};

use crate::tests::config::ClientConfig;
use crate::tests::pass_through::{create_pass_through_account, get_pass_through_note_script};

// TESTS
// ================================================================================================

pub async fn test_output_public_note_ntx(client_config: ClientConfig) -> Result<()> {
    const ASSET_AMOUNT: u64 = 1;

    let (mut client, keystore) = client_config.into_client().await?;
    client.sync_state().await?;

    // let network_account = deploy_counter_contract(&mut client,
    // AccountStorageMode::Network).await?;

    let (sender_account, ..) =
        insert_new_wallet(&mut client, AccountStorageMode::Public, &keystore, RPO_FALCON_SCHEME_ID)
            .await?;
    let pass_through_network_account =
        create_pass_through_account(&mut client, AccountStorageMode::Network).await?;
    let (target_account, ..) =
        insert_new_wallet(&mut client, AccountStorageMode::Public, &keystore, RPO_FALCON_SCHEME_ID)
            .await?;

    // create faucet account to supply sender account with some tokens
    let (faucet_account, ..) = insert_new_fungible_faucet(
        &mut client,
        AccountStorageMode::Public,
        &keystore,
        RPO_FALCON_SCHEME_ID,
    )
    .await?;

    let mint_tx_id =
        mint_and_consume(&mut client, sender_account.id(), faucet_account.id(), NoteType::Public)
            .await;
    wait_for_tx(&mut client, mint_tx_id).await?;

    // Create a note which will be sent to the network pass-through account
    let asset = FungibleAsset::new(faucet_account.id(), ASSET_AMOUNT)?;

    let (pass_through_network_note, pass_through_network_note_details) =
        create_network_pass_through_note(
            sender_account.id(),
            pass_through_network_account.id(),
            target_account.id(),
            asset.into(),
            client.rng(),
        )?;

    let tx_request = TransactionRequestBuilder::new()
        .own_output_notes(vec![OutputNote::Full(pass_through_network_note.clone())])
        .build()?;

    execute_tx_and_sync(&mut client, sender_account.id(), tx_request).await?;

    client.import_notes(&[NoteFile::NoteId(pass_through_network_note.id())]).await?;
    client.sync_state().await?;
    let input_note_record = client.get_input_note(pass_through_network_note.id()).await?.unwrap();
    assert!(matches!(input_note_record.state(), InputNoteState::Committed { .. }));

    let tx_request = TransactionRequestBuilder::new()
        .expected_output_recipients(vec![pass_through_network_note_details.recipient().clone()])
        .build_consume_notes(vec![pass_through_network_note])
        .unwrap();

    let tx_id = client
        .submit_new_transaction(pass_through_network_account.id(), tx_request.clone())
        .await?;

    wait_for_tx(&mut client, tx_id).await?;

    let tx_record = client
        .get_transactions(TransactionFilter::Ids(vec![tx_id]))
        .await?
        .pop()
        .unwrap();

    let output_p2id_note = tx_record.details.output_notes.get_note(0);

    assert_eq!(output_p2id_note.metadata().sender(), pass_through_network_account.id());

    assert_eq!(
        output_p2id_note.metadata().tag(),
        NoteTag::with_account_target(target_account.id())
    );

    assert_eq!(
        output_p2id_note.recipient().expect("note should be full"),
        pass_through_network_note_details.recipient()
    );

    Ok(())
}

// HELPERS
// ================================================================================================

fn create_network_pass_through_note(
    sender: AccountId,
    network_account: AccountId,
    target: AccountId,
    asset: Asset,
    rng: &mut ClientRng,
) -> Result<(Note, NoteDetails)> {
    let pass_through_script = get_pass_through_note_script();

    let asset_word: Word = asset.into();

    let target_recipient = P2idNote::build_recipient(target, rng.draw_word())?;

    let inputs = NoteStorage::new(vec![
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

    let pass_through_recipient = NoteRecipient::new(rng.draw_word(), pass_through_script, inputs);

    let attachment: NoteAttachment =
        NetworkAccountTarget::new(network_account, NoteExecutionHint::Always)?.into();
    let metadata = NoteMetadata::new(sender, NoteType::Public)
        .with_tag(NoteTag::with_account_target(target))
        .with_attachment(attachment);
    let network_note = Note::new(NoteAssets::new(vec![asset])?, metadata, pass_through_recipient);

    let pass_through_note_details =
        NoteDetails::new(NoteAssets::new(vec![asset])?, target_recipient);

    Ok((network_note, pass_through_note_details))
}
