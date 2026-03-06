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
    wait_for_consumable_notes,
    wait_for_node,
    wait_for_tx,
};
use miden_client::transaction::{OutputNote, TransactionRequestBuilder};

use super::agglayer_test_utils::generate_claim_data_for_account;
use crate::tests::config::ClientConfig;

// BRIDGE-IN-OUT TEST
// ================================================================================================

/// Tests the full bridge-in then bridge-out flow using network transactions.
///
/// Setup & config:
/// 1. Creates bridge admin, GER manager (real wallets), bridge account, and agglayer faucet
/// 2. Deploys bridge and agglayer faucet on-chain
/// 3. Registers faucet in bridge via CONFIG_AGG_BRIDGE note (network tx)
/// 4. Creates and deploys destination account (basic wallet)
///
/// Bridge-in:
/// 5. Generates CLAIM proof data by running the foundry test for the destination account
/// 6. Submits UPDATE_GER note from GER manager → consumed by bridge as network tx
/// 7. Submits CLAIM note from GER manager → consumed by agglayer faucet as network tx
/// 8. Destination account consumes the resulting P2ID note to receive bridged tokens
///
/// Bridge-out:
/// 9. Destination account creates B2AGG note with bridged-in assets
/// 10. B2AGG note is consumed by bridge as a network transaction
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
    // SETUP: Create accounts and deploy on-chain
    // ============================================================================================

    // CREATE BRIDGE ADMIN ACCOUNT (owned by bridge admin client)
    let (bridge_admin, ..) = insert_new_wallet(
        &mut bridge_admin_client,
        AccountStorageMode::Private,
        &bridge_admin_keystore,
        RPO_FALCON_SCHEME_ID,
    )
    .await?;
    println!("[bridge_in_out] Bridge admin account created: {:?}", bridge_admin.id());

    // CREATE GER MANAGER ACCOUNT (owned by GER manager client)
    let (ger_manager, ..) = insert_new_wallet(
        &mut ger_manager_client,
        AccountStorageMode::Private,
        &ger_manager_keystore,
        RPO_FALCON_SCHEME_ID,
    )
    .await?;
    println!("[bridge_in_out] GER manager account created: {:?}", ger_manager.id());

    // CREATE BRIDGE ACCOUNT (added to all clients since they all interact with it)
    let bridge_account = create_bridge_account(
        bridge_admin_client.rng().draw_word(),
        bridge_admin.id(),
        ger_manager.id(),
    );
    bridge_admin_client.add_account(&bridge_account, false).await?;
    ger_manager_client.add_account(&bridge_account, false).await?;
    user_client.add_account(&bridge_account, false).await?;
    println!("[bridge_in_out] Bridge account created: {:?}", bridge_account.id());

    // Deploy bridge account on-chain (from bridge admin client)
    let deploy_tx = TransactionRequestBuilder::new().build()?;
    let tx_id = bridge_admin_client
        .submit_new_transaction(bridge_account.id(), deploy_tx)
        .await?;
    wait_for_tx(&mut bridge_admin_client, tx_id).await?;
    println!("[bridge_in_out] Bridge account deployed on-chain");

    // CREATE DESTINATION ACCOUNT (basic wallet, owned by user client)
    // We create a real basic wallet first, then use its AccountId to generate
    // the CLAIM proof data via the foundry test.
    let (destination_account, ..) = insert_new_wallet(
        &mut user_client,
        AccountStorageMode::Public,
        &user_keystore,
        RPO_FALCON_SCHEME_ID,
    )
    .await?;
    println!("[bridge_in_out] Destination account created: {:?}", destination_account.id());

    // Deploy destination account on-chain (from user client)
    let deploy_dest_tx = TransactionRequestBuilder::new().build()?;
    let tx_id = user_client
        .submit_new_transaction(destination_account.id(), deploy_dest_tx)
        .await?;
    wait_for_tx(&mut user_client, tx_id).await?;
    println!("[bridge_in_out] Destination account deployed on-chain");

    // GENERATE CLAIM PROOF DATA via foundry test
    // This runs `forge test` with the destination account's Ethereum address format,
    // generating valid Merkle proofs and leaf data for the CLAIM note.
    let destination_eth_address = EthAddressFormat::from_account_id(destination_account.id());
    println!("[bridge_in_out] Destination eth address: {}", destination_eth_address.to_hex());

    let (proof_data, leaf_data, ger) = generate_claim_data_for_account(destination_account.id());
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

    // CREATE AGGLAYER FAUCET ACCOUNT (added to all clients)
    let origin_token_address = leaf_data.origin_token_address;
    let origin_network = leaf_data.origin_network;
    let scale = 10u8;

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
    println!("[bridge_in_out] Agglayer faucet account created: {:?}", agglayer_faucet.id());

    // Deploy agglayer faucet on-chain (from bridge admin client)
    let deploy_faucet_tx = TransactionRequestBuilder::new().build()?;
    let tx_id = bridge_admin_client
        .submit_new_transaction(agglayer_faucet.id(), deploy_faucet_tx)
        .await?;
    wait_for_tx(&mut bridge_admin_client, tx_id).await?;
    println!("[bridge_in_out] Agglayer faucet deployed on-chain");

    bridge_admin_client.sync_state().await?;
    ger_manager_client.sync_state().await?;
    user_client.sync_state().await?;

    // REGISTER FAUCET IN BRIDGE via CONFIG_AGG_BRIDGE note (submitted from bridge admin)
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
    println!("[bridge_in_out] CONFIG_AGG_BRIDGE note submitted from bridge admin");

    // Wait for bridge to consume the config note as network transaction
    wait_for_blocks(&mut bridge_admin_client, 2).await;
    println!("[bridge_in_out] Waited for bridge to consume CONFIG_AGG_BRIDGE note");

    ger_manager_client.sync_state().await?;

    // ============================================================================================
    // PHASE 1: BRIDGE-IN
    // ============================================================================================

    // CREATE AND SUBMIT UPDATE_GER NOTE (consumed by bridge as network tx)
    let update_ger_note = UpdateGerNote::create(
        ger,
        ger_manager.id(),
        bridge_account.id(),
        ger_manager_client.rng(),
    )?;
    println!("[bridge_in_out] UPDATE_GER note created");

    let tx_request = TransactionRequestBuilder::new()
        .own_output_notes(vec![OutputNote::Full(update_ger_note)])
        .build()?;
    let tx_id = ger_manager_client.submit_new_transaction(ger_manager.id(), tx_request).await?;
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
        agglayer_faucet.id(),
        ger_manager.id(),
        ger_manager_client.rng(),
    )?;
    println!("[bridge_in_out] CLAIM note created");

    let tx_request = TransactionRequestBuilder::new()
        .own_output_notes(vec![OutputNote::Full(claim_note)])
        .build()?;
    let tx_id = ger_manager_client.submit_new_transaction(ger_manager.id(), tx_request).await?;
    wait_for_tx(&mut ger_manager_client, tx_id).await?;
    println!("[bridge_in_out] CLAIM note submitted from GER manager");

    // Wait for agglayer faucet to consume the CLAIM note and create P2ID note.
    // The faucet processes the CLAIM as a network transaction and outputs a P2ID note
    // targeting the destination account. This involves a multi-step chain of network
    // transactions, so we poll with retries rather than waiting a fixed number of blocks.
    let consumable_notes =
        wait_for_consumable_notes(&mut user_client, destination_account.id(), 10).await;
    println!(
        "[bridge_in_out] Found {} consumable notes for destination account",
        consumable_notes.len()
    );

    // Consume all available notes for the destination account (should be the P2ID note)
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
        .get_balance(agglayer_faucet.id())
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
    // Use a portion of the bridged-in amount for the bridge-out
    let bridge_out_amount = 1000u64;
    let destination_network = 0u32;
    // The L1 recipient address where bridged-out assets will be sent on Ethereum
    let l1_destination_address =
        EthAddressFormat::from_hex("0xabcdefabcdefabcdefabcdefabcdefabcdefabcd")
            .expect("valid L1 destination address");

    let bridge_asset: Asset =
        FungibleAsset::new(agglayer_faucet.id(), bridge_out_amount).unwrap().into();
    let b2agg_note = B2AggNote::create(
        destination_network,
        l1_destination_address,
        NoteAssets::new(vec![bridge_asset])?,
        bridge_account.id(),
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
