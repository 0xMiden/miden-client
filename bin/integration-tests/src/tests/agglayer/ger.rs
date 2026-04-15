use anyhow::{Context, Result};
use miden_agglayer::{AggLayerBridge, ExitRoot, UpdateGerNote};
use miden_client::crypto::FeltRng;
use miden_client::note::{
    NetworkAccountTarget,
    Note,
    NoteAssets,
    NoteAttachment,
    NoteExecutionHint,
    NoteMetadata,
    NoteRecipient,
    NoteStorage,
    NoteTag,
    NoteType,
};
use miden_client::testing::common::wait_for_tx;
use miden_client::transaction::TransactionRequestBuilder;

use super::{AgglayerConfig, create_agglayer_clients, setup_core_accounts, wait_for_note_consumed};
use crate::tests::config::ClientConfig;

// TESTS
// ================================================================================================

/// Test GER update flow.
///
/// If `AGGLAYER_ACCOUNTS_DIR` is set, loads pre-deployed accounts from `.mac` files (complete
/// genesis mode). Otherwise, creates all accounts at runtime (empty genesis mode).
pub async fn test_agglayer_update_ger(client_config: ClientConfig) -> Result<()> {
    let agglayer_config = AgglayerConfig::from_env()?;
    let (mut bridge_admin, mut ger_manager, mut user) =
        create_agglayer_clients(&client_config).await?;
    let (_bridge_admin_id, ger_manager_id, bridge_id) = setup_core_accounts(
        agglayer_config.as_ref(),
        &mut bridge_admin,
        &mut ger_manager,
        &mut user,
    )
    .await?;

    // CREATE UPDATE_GER NOTE
    // --------------------------------------------------------------------------------------------
    let ger_bytes: [u8; 32] = rand::random();
    let ger = ExitRoot::from(ger_bytes);
    println!("Submitting UpdateGerNote with random GER: {ger_bytes:02x?}");
    // WORKAROUND: UpdateGerNote::create uses NoteTag(0) which prevents the ntx-builder
    // from discovering the note. Build the note manually with the correct tag.
    let update_ger_note = {
        let storage_values = ger.to_elements().to_vec();
        let note_storage = NoteStorage::new(storage_values)?;
        let serial_num = ger_manager.client.rng().draw_word();
        let recipient = NoteRecipient::new(serial_num, UpdateGerNote::script(), note_storage);

        let attachment = NoteAttachment::from(
            NetworkAccountTarget::new(bridge_id, NoteExecutionHint::Always)
                .map_err(|e| anyhow::anyhow!("{e}"))?,
        );
        let metadata = NoteMetadata::new(ger_manager_id, NoteType::Public)
            .with_tag(NoteTag::with_account_target(bridge_id))
            .with_attachment(attachment);

        Note::new(NoteAssets::new(vec![])?, metadata, recipient)
    };
    let update_ger_note_id = update_ger_note.id();

    let tx_request = TransactionRequestBuilder::new()
        .own_output_notes(vec![update_ger_note])
        .build()?;
    let tx_id = ger_manager.client.submit_new_transaction(ger_manager_id, tx_request).await?;
    wait_for_tx(&mut ger_manager.client, tx_id).await?;

    // WAIT FOR NETWORK ACCOUNT TO PROCESS UPDATE_GER NOTE (via NoteReader)
    // --------------------------------------------------------------------------------------------
    wait_for_note_consumed(&mut ger_manager.client, bridge_id, update_ger_note_id, 30).await?;

    // VERIFY GER HASH WAS STORED IN MAP
    // --------------------------------------------------------------------------------------------
    let updated_bridge_account = ger_manager
        .client
        .test_rpc_api()
        .get_account_details(bridge_id)
        .await?
        .account()
        .cloned()
        .with_context(|| "bridge account details not available")?;

    let is_registered = AggLayerBridge::is_ger_registered(ger, updated_bridge_account)?;
    println!("GER registered: {is_registered}");

    assert!(is_registered, "GER was not registered in the bridge account");

    // LOG ALL CONSUMED NOTES FOR BRIDGE (NoteReader indexer pattern)
    // --------------------------------------------------------------------------------------------
    println!("[NoteReader] All notes consumed by bridge:");
    let update_ger_script = UpdateGerNote::script();
    let mut reader = ger_manager.client.input_note_reader(bridge_id);
    while let Some(note) = reader.next().await? {
        if note.details().script() == &update_ger_script {
            println!(
                "[NoteReader]   note_id={} (UPDATE_GER), state={}, storage={:?}",
                note.id().to_hex(),
                note.state(),
                note.details().storage().items(),
            );
        } else {
            println!("[NoteReader]   note_id={}, state={}", note.id().to_hex(), note.state());
        }
    }

    Ok(())
}
