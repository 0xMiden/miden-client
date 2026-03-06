use anyhow::{Context, Result};
use miden_agglayer::{ExitRoot, UpdateGerNote};
use miden_client::testing::common::{wait_for_blocks, wait_for_tx};
use miden_client::transaction::{OutputNote, TransactionRequestBuilder};

use super::{AgglayerConfig, create_agglayer_clients, setup_core_accounts};
use crate::tests::config::ClientConfig;
use crate::utils::is_ger_registered;

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
    let update_ger_note =
        UpdateGerNote::create(ger, ger_manager_id, bridge_id, ger_manager.client.rng())?;

    let tx_request = TransactionRequestBuilder::new()
        .own_output_notes(vec![OutputNote::Full(update_ger_note)])
        .build()?;
    let tx_id = ger_manager.client.submit_new_transaction(ger_manager_id, tx_request).await?;
    wait_for_tx(&mut ger_manager.client, tx_id).await?;

    // WAIT FOR NETWORK ACCOUNT TO PROCESS UPDATE_GER NOTE
    // --------------------------------------------------------------------------------------------
    wait_for_blocks(&mut ger_manager.client, 2).await;

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

    let is_registered = is_ger_registered(ger, updated_bridge_account)?;
    println!("GER registered: {is_registered}");

    assert!(is_registered, "GER was not registered in the bridge account");

    Ok(())
}
