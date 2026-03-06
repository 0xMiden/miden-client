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
    wait_for_tx,
};
use miden_client::transaction::{OutputNote, TransactionRequestBuilder};

use super::agglayer_test_utils::generate_claim_data_for_account;
use super::{AgglayerConfig, create_agglayer_clients, setup_core_accounts};
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
    let agglayer_config = AgglayerConfig::from_env()?;
    let (mut bridge_admin, mut ger_manager, mut user) =
        create_agglayer_clients(&client_config).await?;
    let core = setup_core_accounts(
        agglayer_config.as_ref(),
        &mut bridge_admin,
        &mut ger_manager,
        &mut user,
    )
    .await?;

    // ============================================================================================
    // SETUP: Destination account (always fresh) + faucet
    // ============================================================================================

    let (destination_account, ..) =
        insert_new_wallet(&mut user.0, AccountStorageMode::Public, &user.1, RPO_FALCON_SCHEME_ID)
            .await?;
    println!("[bridge_in_out] Destination account created: {:?}", destination_account.id());

    let deploy_dest_tx = TransactionRequestBuilder::new().build()?;
    let tx_id = user.0.submit_new_transaction(destination_account.id(), deploy_dest_tx).await?;
    wait_for_tx(&mut user.0, tx_id).await?;
    println!("[bridge_in_out] Destination account deployed on-chain");

    // Set up faucet: either load from genesis or create at runtime
    let (agglayer_faucet_id, origin_token_address, scale) = match &agglayer_config {
        Some(config) => {
            println!("[bridge_in_out] Loading faucet from genesis: {}", config.faucet_id());

            config
                .import_account(config.faucet_id(), &mut bridge_admin.0, &bridge_admin.1)
                .await?;
            config
                .import_account(config.faucet_id(), &mut ger_manager.0, &ger_manager.1)
                .await?;
            config.import_account(config.faucet_id(), &mut user.0, &user.1).await?;

            let origin_token_address = config.faucet_origin_token_address();
            let scale = config.faucet_scale();
            println!(
                "[bridge_in_out] Faucet params - origin_token: {}, scale: {}",
                origin_token_address.to_hex(),
                scale
            );

            (config.faucet_id(), origin_token_address, scale)
        },
        None => {
            let (_, leaf_data_preview, _) =
                generate_claim_data_for_account(destination_account.id(), None);
            let origin_token_address = leaf_data_preview.origin_token_address;
            let origin_network = leaf_data_preview.origin_network;
            let scale = 10u8;

            let agglayer_faucet = create_agglayer_faucet(
                bridge_admin.0.rng().draw_word(),
                "AGG",
                8u8,
                Felt::new(FungibleAsset::MAX_AMOUNT),
                core.bridge_id,
                &origin_token_address,
                origin_network,
                scale,
            );
            println!("[bridge_in_out] Agglayer faucet created: {:?}", agglayer_faucet.id());

            bridge_admin.0.add_account(&agglayer_faucet, false).await?;
            ger_manager.0.add_account(&agglayer_faucet, false).await?;
            user.0.add_account(&agglayer_faucet, false).await?;

            let deploy_faucet_tx = TransactionRequestBuilder::new().build()?;
            let tx_id = bridge_admin
                .0
                .submit_new_transaction(agglayer_faucet.id(), deploy_faucet_tx)
                .await?;
            wait_for_tx(&mut bridge_admin.0, tx_id).await?;
            println!("[bridge_in_out] Agglayer faucet deployed on-chain");

            bridge_admin.0.sync_state().await?;
            ger_manager.0.sync_state().await?;
            user.0.sync_state().await?;

            // Register faucet in bridge via CONFIG_AGG_BRIDGE note
            let config_note = ConfigAggBridgeNote::create(
                agglayer_faucet.id(),
                core.bridge_admin_id,
                core.bridge_id,
                bridge_admin.0.rng(),
            )?;
            let config_output_tx = TransactionRequestBuilder::new()
                .own_output_notes(vec![OutputNote::Full(config_note.clone())])
                .build()?;
            let tx_id = bridge_admin
                .0
                .submit_new_transaction(core.bridge_admin_id, config_output_tx)
                .await?;
            wait_for_tx(&mut bridge_admin.0, tx_id).await?;
            println!("[bridge_in_out] CONFIG_AGG_BRIDGE note submitted");

            wait_for_blocks(&mut bridge_admin.0, 2).await;
            println!("[bridge_in_out] Bridge consumed CONFIG_AGG_BRIDGE note");

            (agglayer_faucet.id(), origin_token_address, scale)
        },
    };

    // ============================================================================================
    // GENERATE CLAIM DATA (always needed - depends on destination account)
    // ============================================================================================

    let (proof_data, leaf_data, ger) =
        generate_claim_data_for_account(destination_account.id(), Some(&origin_token_address));
    println!("[bridge_in_out] Claim data generated via foundry");

    let generated_dest_account_id = leaf_data
        .destination_address
        .to_account_id()
        .expect("generated destination address should be a valid embedded Miden AccountId");
    assert_eq!(
        generated_dest_account_id,
        destination_account.id(),
        "foundry-generated destination must match our wallet's AccountId"
    );

    ger_manager.0.sync_state().await?;

    // ============================================================================================
    // PHASE 1: BRIDGE-IN
    // ============================================================================================

    let update_ger_note =
        UpdateGerNote::create(ger, core.ger_manager_id, core.bridge_id, ger_manager.0.rng())?;
    let tx_request = TransactionRequestBuilder::new()
        .own_output_notes(vec![OutputNote::Full(update_ger_note)])
        .build()?;
    let tx_id = ger_manager.0.submit_new_transaction(core.ger_manager_id, tx_request).await?;
    wait_for_tx(&mut ger_manager.0, tx_id).await?;
    println!("[bridge_in_out] UPDATE_GER note submitted");

    wait_for_blocks(&mut ger_manager.0, 2).await;

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
        core.ger_manager_id,
        ger_manager.0.rng(),
    )?;
    let tx_request = TransactionRequestBuilder::new()
        .own_output_notes(vec![OutputNote::Full(claim_note)])
        .build()?;
    let tx_id = ger_manager.0.submit_new_transaction(core.ger_manager_id, tx_request).await?;
    wait_for_tx(&mut ger_manager.0, tx_id).await?;
    println!("[bridge_in_out] CLAIM note submitted");

    // Wait for agglayer faucet to consume the CLAIM note and create P2ID note.
    // The faucet processes the CLAIM as a network transaction and outputs a P2ID note
    // targeting the destination account. This involves a multi-step chain of network
    // transactions, so we poll with retries rather than waiting a fixed number of blocks.
    let consumable_notes =
        wait_for_consumable_notes(&mut user.0, destination_account.id(), 10).await;
    println!(
        "[bridge_in_out] Found {} consumable notes for destination",
        consumable_notes.len()
    );

    let notes_to_consume: Vec<_> =
        consumable_notes.into_iter().map(|(note, _)| note.try_into().unwrap()).collect();
    let consume_tx = TransactionRequestBuilder::new().build_consume_notes(notes_to_consume)?;
    let tx_id = user.0.submit_new_transaction(destination_account.id(), consume_tx).await?;
    wait_for_tx(&mut user.0, tx_id).await?;
    println!("[bridge_in_out] Destination consumed P2ID note");

    user.0.sync_state().await?;
    let dest_balance = user
        .0
        .account_reader(destination_account.id())
        .get_balance(agglayer_faucet_id)
        .await?;
    println!("[bridge_in_out] Destination balance after bridge-in: {}", dest_balance);
    assert!(dest_balance > 0, "destination should have positive balance after bridge-in");

    println!("[bridge_in_out] Bridge-in phase completed");

    // ============================================================================================
    // PHASE 2: BRIDGE-OUT
    // ============================================================================================

    user.0.sync_state().await?;

    let bridge_out_amount = 1000u64;
    let l1_destination_address =
        EthAddressFormat::from_hex("0xabcdefabcdefabcdefabcdefabcdefabcdefabcd")
            .expect("valid L1 destination address");

    let bridge_asset: Asset =
        FungibleAsset::new(agglayer_faucet_id, bridge_out_amount).unwrap().into();
    let b2agg_note = B2AggNote::create(
        0u32,
        l1_destination_address,
        NoteAssets::new(vec![bridge_asset])?,
        core.bridge_id,
        destination_account.id(),
        user.0.rng(),
    )?;
    println!("[bridge_in_out] B2AGG note created with amount: {}", bridge_out_amount);

    let b2agg_output_tx = TransactionRequestBuilder::new()
        .own_output_notes(vec![OutputNote::Full(b2agg_note)])
        .build()?;
    let tx_id = user.0.submit_new_transaction(destination_account.id(), b2agg_output_tx).await?;
    wait_for_tx(&mut user.0, tx_id).await?;
    println!("[bridge_in_out] B2AGG note submitted");

    wait_for_blocks(&mut user.0, 2).await;
    println!("[bridge_in_out] Test completed successfully");
    Ok(())
}
