use anyhow::{Result, anyhow};
use miden_client::account::{AccountId, AccountStorageMode};
use miden_client::assembly::CodeBuilder;
use miden_client::auth::RPO_FALCON_SCHEME_ID;
use miden_client::crypto::FeltRng;
use miden_client::note::{
    NetworkAccountTarget,
    Note,
    NoteAssets,
    NoteAttachment,
    NoteDetails,
    NoteExecutionHint,
    NoteFile,
    NoteMetadata,
    NoteRecipient,
    NoteStorage,
    NoteTag,
    NoteType,
    P2idNoteStorage,
    StandardNote,
};
use miden_client::store::InputNoteState;
use miden_client::testing::common::{execute_tx_and_sync, insert_new_wallet, wait_for_blocks};
use miden_client::transaction::TransactionRequestBuilder;
use miden_client::{ClientRng, Felt, Word, ZERO};

use crate::tests::config::ClientConfig;
use crate::tests::network_transaction::{
    COUNTER_SLOT_NAME,
    counter_contract_library,
    deploy_counter_contract,
};

const P2ID_EMITTER_SCRIPT: &str = r#"
    use miden::protocol::active_note
    use miden::protocol::output_note
    use miden::core::sys
    use external_contract::counter_contract

    const ERR_STORAGE_LEN="expected 6 note storage items"

    begin
        # drop note arguments
        dropw

        # bump the bank's counter so we can cheaply assert the NTX ran
        call.counter_contract::increment_count

        # load note storage into memory starting at address 0
        push.0 exec.active_note::get_storage
        # => [num_storage_items, storage_ptr]
        eq.6 assert.err=ERR_STORAGE_LEN
        drop
        # => []

        # load RECIPIENT (addresses 0..3), then note_type (4) and tag (5) on top
        padw mem_loadw_le.0
        # => [RECIPIENT]
        mem_load.4
        # => [note_type, RECIPIENT]
        mem_load.5
        # => [tag, note_type, RECIPIENT]

        # emit the public P2ID output note
        exec.output_note::create
        # => [note_idx]
        drop

        exec.sys::truncate_stack
    end
"#;

/// Integration test for issue #1723: a network transaction (NTX) whose input
/// note, when consumed by the node's NTX builder, emits a public P2ID output
/// note.
///
/// Flow:
/// 1. Alice (regular wallet, Public storage) creates a network-targeted note
///    carrying a P2ID recipient digest in its storage.
/// 2. The bank (network-storage-mode account with the counter component) is
///    picked up by the node's NTX builder, executes the note script, and in
///    doing so (a) increments its own counter and (b) emits a public P2ID
///    note targeted at Bob.
/// 3. Verifies the P2ID output note is committed on-chain by importing it
///    by NoteId on a second client and polling for `InputNoteState::Committed`.
///    The counter bump is checked as a secondary signal that the NTX ran.
pub async fn test_ntx_output_public_note(client_config: ClientConfig) -> Result<()> {
    let (mut client, keystore) = client_config.clone().into_client().await?;
    let (mut client_2, keystore_2) = ClientConfig::default()
        .with_rpc_endpoint(client_config.rpc_endpoint())
        .into_client()
        .await?;

    let bank = deploy_counter_contract(&mut client, AccountStorageMode::Network).await?;
    let (alice, ..) = insert_new_wallet(
        &mut client,
        AccountStorageMode::Public,
        &keystore,
        RPO_FALCON_SCHEME_ID,
    )
    .await?;
    let (bob, ..) = insert_new_wallet(
        &mut client_2,
        AccountStorageMode::Public,
        &keystore_2,
        RPO_FALCON_SCHEME_ID,
    )
    .await?;

    // Precompute Bob's P2ID recipient so the same `digest()` used to populate the
    // network note's storage also determines the expected output NoteId.
    let bob_serial_num = client.rng().draw_word();
    let bob_recipient = P2idNoteStorage::new(bob.id()).into_recipient(bob_serial_num);
    let bob_recipient_digest = bob_recipient.digest();
    let expected_output_id =
        NoteDetails::new(NoteAssets::new(vec![])?, bob_recipient).id();

    let network_note = build_emitter_network_note(
        alice.id(),
        bank.id(),
        bob.id(),
        bob_recipient_digest,
        client.rng(),
    )?;

    // `expected_ntx_scripts` registers the P2ID script with the node's NTX
    // script registry if it's not already there (no-op if it is).
    let tx_request = TransactionRequestBuilder::new()
        .own_output_notes(vec![network_note])
        .expected_ntx_scripts(vec![StandardNote::P2ID.script()])
        .build()?;

    execute_tx_and_sync(&mut client, alice.id(), tx_request).await?;

    // `deploy_counter_contract` already ran one increment, so the bank's counter
    // starts at 1; one consumed note bumps it to 2.
    let expected_counter = Word::from([Felt::new(2), ZERO, ZERO, ZERO]);
    for _ in 0..15 {
        wait_for_blocks(&mut client, 1).await;

        let counter_ok = client
            .test_rpc_api()
            .get_account_details(bank.id())
            .await?
            .account()
            .cloned()
            .and_then(|a| a.storage().get_item(&COUNTER_SLOT_NAME).ok())
            .is_some_and(|w| w == expected_counter);
        if !counter_ok {
            continue;
        }

        let _ = client_2
            .import_notes(&[NoteFile::NoteId(expected_output_id)])
            .await;
        client_2.sync_state().await?;
        if let Some(rec) = client_2.get_input_note(expected_output_id).await? {
            if matches!(rec.state(), InputNoteState::Committed { .. }) {
                return Ok(());
            }
        }
    }

    Err(anyhow!(
        "timed out waiting for counter increment and committed P2ID note {expected_output_id}"
    ))
}

// HELPERS
// ================================================================================================

/// Network-targeted note whose script emits a public P2ID note.
///
/// Mirrors `network_transaction::get_network_note_with_script` but stuffs the
/// P2ID recipient digest + note_type + tag into the note storage so the MASM
/// can pass them to `output_note::create`.
fn build_emitter_network_note(
    sender: AccountId,
    network_account: AccountId,
    target: AccountId,
    recipient_digest: Word,
    rng: &mut ClientRng,
) -> Result<Note> {
    let target_ntx = NetworkAccountTarget::new(network_account, NoteExecutionHint::Always)?;
    let attachment: NoteAttachment = target_ntx.into();
    let metadata = NoteMetadata::new(sender, NoteType::Public)
        .with_tag(NoteTag::with_account_target(network_account))
        .with_attachment(attachment);

    let script = CodeBuilder::new()
        .with_dynamically_linked_library(counter_contract_library())?
        .compile_note_script(P2ID_EMITTER_SCRIPT)?;

    // Storage consumed by `P2ID_EMITTER_SCRIPT`:
    //   [0..4] RECIPIENT, [4] note_type, [5] tag
    let storage = NoteStorage::new(vec![
        recipient_digest[0],
        recipient_digest[1],
        recipient_digest[2],
        recipient_digest[3],
        NoteType::Public.into(),
        NoteTag::with_account_target(target).into(),
    ])?;

    let recipient = NoteRecipient::new(rng.draw_word(), script, storage);

    Ok(Note::new(NoteAssets::new(vec![])?, metadata, recipient))
}
