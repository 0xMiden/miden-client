use anyhow::{Context, Result};
use miden_agglayer::{ExitRoot, UpdateGerNote, create_bridge_account};
use miden_client::account::AccountStorageMode;
use miden_client::auth::RPO_FALCON_SCHEME_ID;
use miden_client::crypto::FeltRng;
use miden_client::testing::common::{
    insert_new_wallet,
    wait_for_blocks,
    wait_for_node,
    wait_for_tx,
};
use miden_client::transaction::{OutputNote, TransactionRequestBuilder};

use super::AgglayerConfig;
use crate::tests::config::ClientConfig;
use crate::utils::is_ger_registered;

// TESTS
// ================================================================================================

/// Test GER update flow.
///
/// If `AGGLAYER_ACCOUNTS_DIR` is set, loads pre-deployed accounts from `.mac` files (complete
/// genesis mode). Otherwise, creates all accounts at runtime (empty genesis mode).
pub async fn test_agglayer_update_ger(client_config: ClientConfig) -> Result<()> {
    let (mut client, keystore) = client_config.into_client().await?;
    wait_for_node(&mut client).await;
    client.sync_state().await?;

    let (ger_manager_id, bridge_account_id) = match AgglayerConfig::from_env()? {
        Some(agglayer_config) => {
            agglayer_config.import_into_client(&mut client, &keystore).await?;
            (agglayer_config.ger_manager_id(), agglayer_config.bridge_id())
        },
        None => {
            // Create accounts at runtime
            let (bridge_admin, ..) = insert_new_wallet(
                &mut client,
                AccountStorageMode::Private,
                &keystore,
                RPO_FALCON_SCHEME_ID,
            )
            .await?;

            let (ger_manager, ..) = insert_new_wallet(
                &mut client,
                AccountStorageMode::Private,
                &keystore,
                RPO_FALCON_SCHEME_ID,
            )
            .await?;

            let bridge_account = create_bridge_account(
                client.rng().draw_word(),
                bridge_admin.id(),
                ger_manager.id(),
            );
            client.add_account(&bridge_account, false).await?;

            // Deploy the bridge account on-chain with a no-op transaction
            let deploy_tx_request = TransactionRequestBuilder::new().build()?;
            let tx_id =
                client.submit_new_transaction(bridge_account.id(), deploy_tx_request).await?;
            wait_for_tx(&mut client, tx_id).await?;

            (ger_manager.id(), bridge_account.id())
        },
    };

    // CREATE UPDATE_GER NOTE
    // --------------------------------------------------------------------------------------------
    let ger_bytes: [u8; 32] = rand::random();
    let ger = ExitRoot::from(ger_bytes);
    let update_ger_note =
        UpdateGerNote::create(ger, ger_manager_id, bridge_account_id, client.rng())?;

    let tx_request = TransactionRequestBuilder::new()
        .own_output_notes(vec![OutputNote::Full(update_ger_note)])
        .build()?;
    let tx_id = client.submit_new_transaction(ger_manager_id, tx_request).await?;
    wait_for_tx(&mut client, tx_id).await?;

    // WAIT FOR NETWORK ACCOUNT TO PROCESS UPDATE_GER NOTE
    // --------------------------------------------------------------------------------------------
    wait_for_blocks(&mut client, 2).await;

    // VERIFY GER HASH WAS STORED IN MAP
    // --------------------------------------------------------------------------------------------
    let updated_bridge_account = client
        .test_rpc_api()
        .get_account_details(bridge_account_id)
        .await?
        .account()
        .cloned()
        .with_context(|| "bridge account details not available")?;

    let is_registered = is_ger_registered(ger, updated_bridge_account)?;

    assert!(is_registered, "GER was not registered in the bridge account");

    Ok(())
}
