use anyhow::{Context, Result};
use miden_agglayer::{AggLayerBridge, ExitRoot, UpdateGerNote, create_bridge_account};
use miden_client::account::AccountStorageMode;
use miden_client::auth::RPO_FALCON_SCHEME_ID;
use miden_client::crypto::{FeltRng, Rpo256};
use miden_client::testing::common::{
    insert_new_wallet,
    wait_for_blocks,
    wait_for_node,
    wait_for_tx,
};
use miden_client::transaction::{OutputNote, TransactionRequestBuilder};
use miden_client::{Felt, ONE, Word, ZERO};

use crate::tests::config::ClientConfig;

// TESTS
// ================================================================================================

pub async fn test_agglayer_update_ger(client_config: ClientConfig) -> Result<()> {
    let (mut client, keystore) = client_config.into_client().await?;
    wait_for_node(&mut client).await;
    client.sync_state().await?;

    // CREATE BRIDGE ADMIN ACCOUNT (not used in this test, but distinct from GER manager)
    // --------------------------------------------------------------------------------------------
    let (bridge_admin, ..) = insert_new_wallet(
        &mut client,
        AccountStorageMode::Private,
        &keystore,
        RPO_FALCON_SCHEME_ID,
    )
    .await?;

    // CREATE GER MANAGER ACCOUNT (NOTE SENDER)
    // --------------------------------------------------------------------------------------------
    let (ger_manager, ..) = insert_new_wallet(
        &mut client,
        AccountStorageMode::Private,
        &keystore,
        RPO_FALCON_SCHEME_ID,
    )
    .await?;

    // CREATE BRIDGE ACCOUNT
    // --------------------------------------------------------------------------------------------
    let bridge_account =
        create_bridge_account(client.rng().draw_word(), bridge_admin.id(), ger_manager.id());
    client.add_account(&bridge_account, false).await?;

    // Deploy the bridge account on-chain with a no-op transaction
    let deploy_tx_request = TransactionRequestBuilder::new().build()?;
    let tx_id = client.submit_new_transaction(bridge_account.id(), deploy_tx_request).await?;
    wait_for_tx(&mut client, tx_id).await?;

    // CREATE UPDATE_GER NOTE
    // --------------------------------------------------------------------------------------------
    let ger_bytes: [u8; 32] = [
        0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77,
        0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66,
        0x77, 0x88,
    ];
    let ger = ExitRoot::from(ger_bytes);
    let update_ger_note =
        UpdateGerNote::create(ger, ger_manager.id(), bridge_account.id(), client.rng())?;

    let tx_request = TransactionRequestBuilder::new()
        .own_output_notes(vec![OutputNote::Full(update_ger_note)])
        .build()?;
    let tx_id = client.submit_new_transaction(ger_manager.id(), tx_request).await?;
    wait_for_tx(&mut client, tx_id).await?;

    // WAIT FOR NETWORK ACCOUNT TO PROCESS UPDATE_GER NOTE
    // --------------------------------------------------------------------------------------------
    wait_for_blocks(&mut client, 2).await;

    // VERIFY GER HASH WAS STORED IN MAP
    // --------------------------------------------------------------------------------------------
    let updated_bridge_account = client
        .test_rpc_api()
        .get_account_details(bridge_account.id())
        .await?
        .account()
        .cloned()
        .with_context(|| "bridge account details not available")?;

    // Compute the expected GER hash: rpo256::merge(GER_UPPER, GER_LOWER)
    let mut ger_lower: [Felt; 4] = ger.to_elements()[0..4].try_into().unwrap();
    let mut ger_upper: [Felt; 4] = ger.to_elements()[4..8].try_into().unwrap();
    // Elements are reversed: rpo256::merge treats stack as if loaded BE from memory
    // The following will produce matching hashes:
    // Rust
    // Hasher::merge(&[a, b, c, d], &[e, f, g, h])
    // MASM
    // rpo256::merge(h, g, f, e, d, c, b, a)
    ger_lower.reverse();
    ger_upper.reverse();
    let ger_hash = Rpo256::merge(&[ger_upper.into(), ger_lower.into()]);

    let ger_storage_slot = AggLayerBridge::ger_map_slot_name();
    let stored_value = updated_bridge_account
        .storage()
        .get_map_item(ger_storage_slot, ger_hash)
        .expect("GER hash should be stored in the map");
    let expected_value: Word = [ONE, ZERO, ZERO, ZERO].into();
    assert_eq!(stored_value, expected_value, "GER hash should map to [1, 0, 0, 0]");

    Ok(())
}
