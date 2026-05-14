use std::sync::Arc;

use anyhow::{Result, anyhow};
use miden_client::account::{AccountId, AccountStorageMode};
use miden_client::assembly::{CodeBuilder, SourceManagerSync};
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

// Layout of the network note's storage as consumed by `P2ID_EMITTER_SCRIPT`:
//   [ 0.. 4) SERIAL_NUM       (4 felts) — for the output P2ID note
//   [ 4.. 8) SCRIPT_ROOT      (4 felts) — root of the P2ID note script
//   [ 8..10) P2ID storage     (2 felts) — target account suffix, prefix
//   [10]     note_type        (Public)
//   [11]     tag              (NoteTag::with_account_target(target))
const EMITTER_NOTE_NUM_STORAGE_ITEMS: u32 = 12;

const P2ID_EMITTER_SCRIPT: &str = r#"
    use miden::protocol::active_note
    use miden::protocol::note
    use miden::protocol::output_note
    use miden::core::sys
    use external_contract::counter_contract

    const ERR_STORAGE_LEN="expected 12 note storage items"

    @note_script
    pub proc main
        # drop note arguments
        dropw

        # bump the bank's counter so we can cheaply assert the NTX ran
        call.counter_contract::increment_count

        # load note storage into memory starting at address 0
        push.0 exec.active_note::get_storage
        # => [num_storage_items, storage_ptr]
        eq.12 assert.err=ERR_STORAGE_LEN
        drop
        # => []

        # Build the recipient via the canonical helper. This is the same path
        # used by `MINT.masm` for public output notes: it computes the recipient
        # digest AND populates the advice map with the entries the kernel's
        # `note::before_created` event handler reads when materializing the
        # public output note (recipient_digest -> [sn_script_hash, storage_commitment],
        # sn_script_hash, sn_hash, storage_commitment -> storage items).
        # Without this, the kernel can only see the recipient digest and aborts
        # the NTX with `PublicNoteMissingDetails`.
        padw mem_loadw_le.4
        # => [SCRIPT_ROOT]
        padw mem_loadw_le.0
        # => [SERIAL_NUM, SCRIPT_ROOT]
        push.2 push.8
        # => [storage_ptr=8, num_storage_items=2, SERIAL_NUM, SCRIPT_ROOT]

        exec.note::build_recipient
        # => [RECIPIENT]

        mem_load.10
        # => [note_type, RECIPIENT]
        mem_load.11
        # => [tag, note_type, RECIPIENT]

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
/// 1. Alice (regular wallet, Public storage) creates a network-targeted note carrying the
///    components of a P2ID recipient (serial num, script root, P2ID storage) in its storage.
/// 2. The bank (network-storage-mode account with the counter component) is picked up by the node's
///    NTX builder, executes the note script, and in doing so (a) increments its own counter and (b)
///    emits a public P2ID note targeted at Bob.
/// 3. Verifies the P2ID output note is committed on-chain by importing it by NoteId on a second
///    client and polling for `InputNoteState::Committed`. The counter bump is checked as a
///    secondary signal that the NTX ran.
pub async fn test_ntx_output_public_note(client_config: ClientConfig) -> Result<()> {
    let (mut client, keystore) = client_config.clone().into_client().await?;
    let (mut client_2, keystore_2) = ClientConfig::default()
        .with_rpc_endpoint(client_config.rpc_endpoint())
        .into_client()
        .await?;

    let bank = deploy_counter_contract(&mut client, AccountStorageMode::Network).await?;
    let (alice, ..) =
        insert_new_wallet(&mut client, AccountStorageMode::Public, &keystore, RPO_FALCON_SCHEME_ID)
            .await?;
    let (bob, ..) = insert_new_wallet(
        &mut client_2,
        AccountStorageMode::Public,
        &keystore_2,
        RPO_FALCON_SCHEME_ID,
    )
    .await?;

    // Precompute Bob's P2ID recipient. The same components used to populate
    // the network note's storage determine the expected output NoteId.
    let bob_serial_num = client.rng().draw_word();
    let bob_recipient = P2idNoteStorage::new(bob.id()).into_recipient(bob_serial_num);
    let expected_output_id = NoteDetails::new(NoteAssets::new(vec![])?, bob_recipient.clone()).id();

    let network_note = build_emitter_network_note(
        alice.id(),
        bank.id(),
        bob.id(),
        &bob_recipient,
        client.source_manager(),
        client.rng(),
    )?;

    let tx_request =
        TransactionRequestBuilder::new().own_output_notes(vec![network_note]).build()?;

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

        let _ = client_2.import_notes(&[NoteFile::NoteId(expected_output_id)]).await;
        client_2.sync_state().await?;
        if let Some(rec) = client_2.get_input_note(expected_output_id).await?
            && matches!(rec.state(), InputNoteState::Committed { .. })
        {
            return Ok(());
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
/// Stuffs the components of the output P2ID's recipient (serial num, script
/// root, P2ID storage) plus the output's note_type and tag into the network
/// note's own storage so the MASM can pass them to `note::build_recipient`
/// and `output_note::create`.
fn build_emitter_network_note(
    sender: AccountId,
    network_account: AccountId,
    target: AccountId,
    output_recipient: &NoteRecipient,
    source_manager: Arc<dyn SourceManagerSync>,
    rng: &mut ClientRng,
) -> Result<Note> {
    let target_ntx = NetworkAccountTarget::new(network_account, NoteExecutionHint::Always)?;
    let attachment: NoteAttachment = target_ntx.into();
    let metadata = NoteMetadata::new(sender, NoteType::Public)
        .with_tag(NoteTag::with_account_target(network_account))
        .with_attachment(attachment);

    let script = CodeBuilder::with_source_manager(source_manager.clone())
        .with_dynamically_linked_library(counter_contract_library(source_manager))?
        .compile_note_script(P2ID_EMITTER_SCRIPT)?;

    let serial_num = output_recipient.serial_num();
    let script_root = output_recipient.script().root();
    let output_storage_items = output_recipient.storage().items();

    let mut storage_values: Vec<Felt> = Vec::with_capacity(EMITTER_NOTE_NUM_STORAGE_ITEMS as usize);
    storage_values.extend_from_slice(serial_num.as_elements());
    storage_values.extend_from_slice(script_root.as_elements());
    storage_values.extend_from_slice(output_storage_items);
    storage_values.push(NoteType::Public.into());
    storage_values.push(NoteTag::with_account_target(target).into());
    debug_assert_eq!(storage_values.len(), EMITTER_NOTE_NUM_STORAGE_ITEMS as usize);

    let storage = NoteStorage::new(storage_values)?;
    let recipient = NoteRecipient::new(rng.draw_word(), script, storage);

    Ok(Note::new(NoteAssets::new(vec![])?, metadata, recipient))
}
