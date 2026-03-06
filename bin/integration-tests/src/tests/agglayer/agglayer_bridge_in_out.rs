//! Agglayer bridge-in and bridge-out end-to-end integration test.
//!
//! Exercises the full bridge lifecycle as network transaction flows:
//!
//! Setup & config:
//! 1. Deploy bridge and agglayer faucet accounts
//! 2. Register faucet in bridge via CONFIG_AGG_BRIDGE note
//! 3. Create and deploy destination account (basic wallet)
//!
//! Bridge-in:
//! 4. Generate CLAIM proof data by calling foundry test for the destination account
//! 5. Submit UPDATE_GER note → consumed by bridge as network transaction
//! 6. Submit CLAIM note → consumed by agglayer faucet as network transaction
//! 7. Destination account consumes the resulting P2ID note
//!
//! Bridge-out:
//! 8. Submit B2AGG note from destination account → consumed by bridge as network transaction

extern crate alloc;

use alloc::vec;

use anyhow::Result;
use miden_agglayer::{
    B2AggNote,
    ClaimNoteStorage,
    ConfigAggBridgeNote,
    EthAddressFormat,
    UpdateGerNote,
    create_agglayer_faucet,
    create_bridge_account,
    create_claim_note,
};
use miden_client::Felt;
use miden_client::account::AccountStorageMode;
use miden_client::asset::{Asset, FungibleAsset};
use miden_client::auth::RPO_FALCON_SCHEME_ID;
use miden_client::crypto::FeltRng;
use miden_client::note::NoteAssets;
use miden_client::testing::common::{
    insert_new_wallet,
    wait_for_blocks,
    wait_for_node,
    wait_for_tx,
};
use miden_client::transaction::{OutputNote, TransactionRequestBuilder};

use super::AgglayerConfig;
use super::agglayer_test_utils::generate_claim_data_for_account;
use crate::tests::config::ClientConfig;

// BRIDGE-IN-OUT TEST
// ================================================================================================

/// Tests the full bridge-in then bridge-out flow using network transactions.
///
/// If `AGGLAYER_ACCOUNTS_DIR` is set, loads bridge admin, GER manager, bridge, and faucet
/// from genesis files. Otherwise, creates all accounts at runtime.
///
/// In both modes, a fresh destination account is created for the user.
pub async fn test_bridge_in_out(client_config: ClientConfig) -> Result<()> {
    // Create separate clients for each entity, each with its own keystore for signing.
    // All clients share the same RPC endpoint but have independent stores and keystores.

    // Bridge admin client: submits CONFIG_AGG_BRIDGE notes
    let (mut bridge_admin_client, bridge_admin_keystore) =
        client_config.clone().into_client().await?;
    wait_for_node(&mut bridge_admin_client).await;
    bridge_admin_client.sync_state().await?;
    println!("[bridge_in_out] Bridge admin client initialized and synced");

    // GER manager client: submits UPDATE_GER and CLAIM notes
    let (mut ger_manager_client, ger_manager_keystore) = ClientConfig::default()
        .with_rpc_endpoint(client_config.rpc_endpoint())
        .into_client()
        .await?;
    println!("[bridge_in_out] GER manager client initialized and synced");

    // User client: owns the destination account, consumes P2ID notes, submits B2AGG notes
    let (mut user_client, user_keystore) = ClientConfig::default()
        .with_rpc_endpoint(client_config.rpc_endpoint())
        .into_client()
        .await?;
    println!("[bridge_in_out] User client initialized and synced");

    // ============================================================================================
    // SETUP: Create or load accounts
    // ============================================================================================

    // Destination account is always created fresh (user-specific, not part of genesis)
    let (destination_account, ..) = insert_new_wallet(
        &mut user_client,
        AccountStorageMode::Public,
        &user_keystore,
        RPO_FALCON_SCHEME_ID,
    )
    .await?;
    println!("[bridge_in_out] Destination account created: {:?}", destination_account.id());

    let deploy_dest_tx = TransactionRequestBuilder::new().build()?;
    let tx_id = user_client
        .submit_new_transaction(destination_account.id(), deploy_dest_tx)
        .await?;
    wait_for_tx(&mut user_client, tx_id).await?;
    println!("[bridge_in_out] Destination account deployed on-chain");

    // Branch: load from genesis files or create at runtime
    let (_bridge_admin_id, ger_manager_id, bridge_account_id, agglayer_faucet_id, origin_token_address, _origin_network, scale) =
        match AgglayerConfig::from_env()? {
            Some(config) => {
                println!("[bridge_in_out] Loading agglayer accounts from genesis files");
                println!("[bridge_in_out]   bridge admin:  {}", config.bridge_admin_id());
                println!("[bridge_in_out]   GER manager:   {}", config.ger_manager_id());
                println!("[bridge_in_out]   bridge:        {}", config.bridge_id());
                println!("[bridge_in_out]   faucet:        {}", config.faucet_id());

                // Import bridge admin + keys into bridge_admin_client
                config
                    .import_account(
                        config.bridge_admin_id(),
                        &mut bridge_admin_client,
                        &bridge_admin_keystore,
                    )
                    .await?;

                // Import GER manager + keys into ger_manager_client
                config
                    .import_account(
                        config.ger_manager_id(),
                        &mut ger_manager_client,
                        &ger_manager_keystore,
                    )
                    .await?;

                // Import bridge and faucet (no secret keys) into all 3 clients
                for client in
                    [&mut bridge_admin_client, &mut ger_manager_client, &mut user_client]
                {
                    // Use a dummy keystore - these accounts have no keys (NoAuth)
                    let dummy_keystore = &bridge_admin_keystore;
                    config.import_account(config.bridge_id(), client, dummy_keystore).await?;
                    config.import_account(config.faucet_id(), client, dummy_keystore).await?;
                }

                let origin_token_address = config.faucet_origin_token_address();
                let origin_network = config.faucet_origin_network();
                let scale = config.faucet_scale();
                println!(
                    "[bridge_in_out] Faucet params - origin_token: {}, network: {}, scale: {}",
                    origin_token_address.to_hex(),
                    origin_network,
                    scale
                );

                (
                    config.bridge_admin_id(),
                    config.ger_manager_id(),
                    config.bridge_id(),
                    config.faucet_id(),
                    origin_token_address,
                    origin_network,
                    scale,
                )
            },
            None => {
                println!("[bridge_in_out] Creating agglayer accounts at runtime");

                // CREATE BRIDGE ADMIN ACCOUNT
                let (bridge_admin, ..) = insert_new_wallet(
                    &mut bridge_admin_client,
                    AccountStorageMode::Private,
                    &bridge_admin_keystore,
                    RPO_FALCON_SCHEME_ID,
                )
                .await?;
                println!("[bridge_in_out] Bridge admin account created: {:?}", bridge_admin.id());

                // CREATE GER MANAGER ACCOUNT
                let (ger_manager, ..) = insert_new_wallet(
                    &mut ger_manager_client,
                    AccountStorageMode::Private,
                    &ger_manager_keystore,
                    RPO_FALCON_SCHEME_ID,
                )
                .await?;
                println!(
                    "[bridge_in_out] GER manager account created: {:?}",
                    ger_manager.id()
                );

                // CREATE BRIDGE ACCOUNT
                let bridge_account = create_bridge_account(
                    bridge_admin_client.rng().draw_word(),
                    bridge_admin.id(),
                    ger_manager.id(),
                );
                bridge_admin_client.add_account(&bridge_account, false).await?;
                ger_manager_client.add_account(&bridge_account, false).await?;
                user_client.add_account(&bridge_account, false).await?;
                println!(
                    "[bridge_in_out] Bridge account created: {:?}",
                    bridge_account.id()
                );

                // Deploy bridge account on-chain
                let deploy_tx = TransactionRequestBuilder::new().build()?;
                let tx_id = bridge_admin_client
                    .submit_new_transaction(bridge_account.id(), deploy_tx)
                    .await?;
                wait_for_tx(&mut bridge_admin_client, tx_id).await?;
                println!("[bridge_in_out] Bridge account deployed on-chain");

                // Generate claim data to determine origin_token_address
                let destination_eth_address =
                    EthAddressFormat::from_account_id(destination_account.id());
                println!(
                    "[bridge_in_out] Destination eth address: {}",
                    destination_eth_address.to_hex()
                );

                let (_, leaf_data_preview, _) =
                    generate_claim_data_for_account(destination_account.id(), None);
                let origin_token_address = leaf_data_preview.origin_token_address;
                let origin_network = leaf_data_preview.origin_network;
                let scale = 10u8;

                // CREATE AGGLAYER FAUCET ACCOUNT
                let agglayer_faucet = create_agglayer_faucet(
                    bridge_admin_client.rng().draw_word(),
                    "AGG",
                    8u8,
                    Felt::new(FungibleAsset::MAX_AMOUNT),
                    bridge_account.id(),
                    &origin_token_address,
                    origin_network,
                    scale,
                );
                bridge_admin_client.add_account(&agglayer_faucet, false).await?;
                ger_manager_client.add_account(&agglayer_faucet, false).await?;
                user_client.add_account(&agglayer_faucet, false).await?;
                println!(
                    "[bridge_in_out] Agglayer faucet account created: {:?}",
                    agglayer_faucet.id()
                );

                // Deploy agglayer faucet on-chain
                let deploy_faucet_tx = TransactionRequestBuilder::new().build()?;
                let tx_id = bridge_admin_client
                    .submit_new_transaction(agglayer_faucet.id(), deploy_faucet_tx)
                    .await?;
                wait_for_tx(&mut bridge_admin_client, tx_id).await?;
                println!("[bridge_in_out] Agglayer faucet deployed on-chain");

                bridge_admin_client.sync_state().await?;
                ger_manager_client.sync_state().await?;
                user_client.sync_state().await?;

                // REGISTER FAUCET IN BRIDGE via CONFIG_AGG_BRIDGE note
                let config_note = ConfigAggBridgeNote::create(
                    agglayer_faucet.id(),
                    bridge_admin.id(),
                    bridge_account.id(),
                    bridge_admin_client.rng(),
                )?;
                println!("[bridge_in_out] CONFIG_AGG_BRIDGE note created");

                let config_output_tx = TransactionRequestBuilder::new()
                    .own_output_notes(vec![OutputNote::Full(config_note.clone())])
                    .build()?;
                let tx_id = bridge_admin_client
                    .submit_new_transaction(bridge_admin.id(), config_output_tx)
                    .await?;
                wait_for_tx(&mut bridge_admin_client, tx_id).await?;
                println!("[bridge_in_out] CONFIG_AGG_BRIDGE note submitted");

                wait_for_blocks(&mut bridge_admin_client, 2).await;
                println!("[bridge_in_out] Waited for bridge to consume CONFIG_AGG_BRIDGE note");

                (
                    bridge_admin.id(),
                    ger_manager.id(),
                    bridge_account.id(),
                    agglayer_faucet.id(),
                    origin_token_address,
                    origin_network,
                    scale,
                )
            },
        };

    // ============================================================================================
    // GENERATE CLAIM DATA (always needed - depends on destination account)
    // ============================================================================================

    let (proof_data, leaf_data, ger) = generate_claim_data_for_account(
        destination_account.id(),
        Some(&origin_token_address),
    );
    println!("[bridge_in_out] Claim data generated via foundry for destination account");

    // Verify the generated data targets our destination account
    let generated_dest_account_id = leaf_data
        .destination_address
        .to_account_id()
        .expect("generated destination address should be a valid embedded Miden AccountId");
    assert_eq!(
        generated_dest_account_id,
        destination_account.id(),
        "foundry-generated destination must match our wallet's AccountId"
    );

    ger_manager_client.sync_state().await?;

    // ============================================================================================
    // PHASE 1: BRIDGE-IN
    // ============================================================================================

    // CREATE AND SUBMIT UPDATE_GER NOTE (consumed by bridge as network tx)
    let update_ger_note = UpdateGerNote::create(
        ger,
        ger_manager_id,
        bridge_account_id,
        ger_manager_client.rng(),
    )?;
    println!("[bridge_in_out] UPDATE_GER note created");

    let tx_request = TransactionRequestBuilder::new()
        .own_output_notes(vec![OutputNote::Full(update_ger_note)])
        .build()?;
    let tx_id = ger_manager_client.submit_new_transaction(ger_manager_id, tx_request).await?;
    wait_for_tx(&mut ger_manager_client, tx_id).await?;
    println!("[bridge_in_out] UPDATE_GER note submitted from GER manager");

    // Wait for bridge to consume the UPDATE_GER note
    wait_for_blocks(&mut ger_manager_client, 2).await;
    println!("[bridge_in_out] Waited for bridge to consume UPDATE_GER note");

    // CREATE AND SUBMIT CLAIM NOTE (consumed by agglayer faucet as network tx)
    let miden_claim_amount = leaf_data
        .amount
        .scale_to_token_amount(scale as u32)
        .expect("amount should scale successfully");
    println!("[bridge_in_out] Miden claim amount: {:?}", miden_claim_amount);

    let claim_inputs = ClaimNoteStorage {
        proof_data,
        leaf_data,
        miden_claim_amount,
    };

    let claim_note = create_claim_note(
        claim_inputs,
        agglayer_faucet_id,
        ger_manager_id,
        ger_manager_client.rng(),
    )?;
    println!("[bridge_in_out] CLAIM note created");

    let tx_request = TransactionRequestBuilder::new()
        .own_output_notes(vec![OutputNote::Full(claim_note)])
        .build()?;
    let tx_id = ger_manager_client.submit_new_transaction(ger_manager_id, tx_request).await?;
    wait_for_tx(&mut ger_manager_client, tx_id).await?;
    println!("[bridge_in_out] CLAIM note submitted from GER manager");

    // Wait for agglayer faucet to consume the CLAIM note and create P2ID note
    wait_for_blocks(&mut ger_manager_client, 2).await;
    println!("[bridge_in_out] Waited for agglayer faucet to consume CLAIM note");

    // CONSUME P2ID NOTE WITH DESTINATION ACCOUNT
    user_client.sync_state().await?;

    let consumable_notes = user_client.get_consumable_notes(Some(destination_account.id())).await?;
    println!(
        "[bridge_in_out] Found {} consumable notes for destination account",
        consumable_notes.len()
    );
    assert!(
        !consumable_notes.is_empty(),
        "destination account should have at least one consumable P2ID note after bridge-in"
    );

    let notes_to_consume: Vec<_> =
        consumable_notes.into_iter().map(|(note, _)| note.try_into().unwrap()).collect();
    let consume_tx = TransactionRequestBuilder::new().build_consume_notes(notes_to_consume)?;
    let tx_id = user_client.submit_new_transaction(destination_account.id(), consume_tx).await?;
    wait_for_tx(&mut user_client, tx_id).await?;
    println!("[bridge_in_out] Destination account consumed P2ID note");

    // Verify destination account has the bridged assets
    user_client.sync_state().await?;
    let dest_balance = user_client
        .account_reader(destination_account.id())
        .get_balance(agglayer_faucet_id)
        .await?;
    println!("[bridge_in_out] Destination account balance after bridge-in: {}", dest_balance);
    assert!(
        dest_balance > 0,
        "destination account should have a positive balance after consuming the P2ID note"
    );

    println!("[bridge_in_out] Bridge-in phase completed successfully");

    // ============================================================================================
    // PHASE 2: BRIDGE-OUT
    // ============================================================================================

    user_client.sync_state().await?;

    // CREATE AND SUBMIT B2AGG NOTE (bridge-out from destination account)
    let bridge_out_amount = 1000u64;
    let destination_network = 0u32;
    let l1_destination_address =
        EthAddressFormat::from_hex("0xabcdefabcdefabcdefabcdefabcdefabcdefabcd")
            .expect("valid L1 destination address");

    let bridge_asset: Asset =
        FungibleAsset::new(agglayer_faucet_id, bridge_out_amount).unwrap().into();
    let b2agg_note = B2AggNote::create(
        destination_network,
        l1_destination_address,
        NoteAssets::new(vec![bridge_asset])?,
        bridge_account_id,
        destination_account.id(),
        user_client.rng(),
    )?;
    println!("[bridge_in_out] B2AGG note created with amount: {}", bridge_out_amount);

    let b2agg_output_tx = TransactionRequestBuilder::new()
        .own_output_notes(vec![OutputNote::Full(b2agg_note)])
        .build()?;
    let tx_id = user_client
        .submit_new_transaction(destination_account.id(), b2agg_output_tx)
        .await?;
    wait_for_tx(&mut user_client, tx_id).await?;
    println!("[bridge_in_out] B2AGG note submitted from destination account");

    // Wait for bridge to consume the B2AGG note as network transaction
    wait_for_blocks(&mut user_client, 2).await;
    println!("[bridge_in_out] Waited for bridge to consume B2AGG note");

    println!("[bridge_in_out] Test completed successfully");
    Ok(())
}
