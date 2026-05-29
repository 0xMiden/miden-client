use anyhow::{Context, Result};
use miden_client::account::AccountType;
use miden_client::asset::FungibleAsset;
use miden_client::auth::RPO_FALCON_SCHEME_ID;
use miden_client::note::{Note, NoteDetails, NoteType, PswapNote};
use miden_client::store::NoteFilter;
use miden_client::testing::common::*;
use miden_client::transaction::{PswapTransactionData, TransactionRequestBuilder};
use tracing::info;

use crate::tests::config::ClientConfig;

// PSWAP FULL FILL ONCHAIN
// ================================================================================================

/// Verifies an end-to-end PSWAP full-fill flow against a real node:
/// Alice creates a public PSWAP, Bob discovers it via the discovery tag, Bob fully fills it, and
/// both parties end up with the expected balances after consuming the resulting payback note.
///
/// The PSWAP consume MASM emits a payback note with a word-sized attachment (see
/// `add_word_attachment` in `standards/notes/pswap.masm`); Alice fetches that payback note via sync
/// and consumes it.
pub async fn test_pswap_full_fill_onchain(client_config: ClientConfig) -> Result<()> {
    const OFFERED_AMOUNT: u64 = 100;
    const REQUESTED_AMOUNT: u64 = 50;

    let (mut alice_client, alice_authenticator) = client_config.clone().into_client().await?;
    wait_for_node(&mut alice_client).await;
    let (mut bob_client, bob_authenticator) = ClientConfig::default()
        .with_rpc_endpoint(client_config.rpc_endpoint())
        .into_client()
        .await?;

    alice_client.sync_state().await?;
    bob_client.sync_state().await?;

    let (alice_account, ..) = insert_new_wallet(
        &mut alice_client,
        AccountType::Private,
        &alice_authenticator,
        RPO_FALCON_SCHEME_ID,
    )
    .await?;
    let (bob_account, ..) = insert_new_wallet(
        &mut bob_client,
        AccountType::Private,
        &bob_authenticator,
        RPO_FALCON_SCHEME_ID,
    )
    .await?;

    let (btc_faucet_account, _) = insert_new_fungible_faucet(
        &mut alice_client,
        AccountType::Private,
        &alice_authenticator,
        RPO_FALCON_SCHEME_ID,
    )
    .await?;
    let (eth_faucet_account, _) = insert_new_fungible_faucet(
        &mut bob_client,
        AccountType::Private,
        &bob_authenticator,
        RPO_FALCON_SCHEME_ID,
    )
    .await?;

    let tx_id = mint_and_consume(
        &mut alice_client,
        alice_account.id(),
        btc_faucet_account.id(),
        NoteType::Public,
    )
    .await;
    wait_for_tx(&mut alice_client, tx_id).await?;
    let tx_id = mint_and_consume(
        &mut bob_client,
        bob_account.id(),
        eth_faucet_account.id(),
        NoteType::Public,
    )
    .await;
    wait_for_tx(&mut bob_client, tx_id).await?;

    let offered_asset = FungibleAsset::new(btc_faucet_account.id(), OFFERED_AMOUNT)?;
    let requested_asset = FungibleAsset::new(eth_faucet_account.id(), REQUESTED_AMOUNT)?;

    info!("Executing PSWAP create transaction");
    let tx_request = TransactionRequestBuilder::new().build_pswap_create(
        &PswapTransactionData::new(alice_account.id(), offered_asset, requested_asset),
        NoteType::Public,
        NoteType::Public,
        None,
        alice_client.rng(),
    )?;

    let pswap_note = tx_request.expected_output_own_notes()[0].clone();
    execute_tx_and_sync(&mut alice_client, alice_account.id(), tx_request).await?;

    // Subscribe bob_client to the PSWAP discovery tag so it can pick up the public note.
    let pswap_tag = PswapNote::create_tag(NoteType::Public, &offered_asset, &requested_asset);
    info!(tag = %pswap_tag, "Adding PSWAP discovery tag to client 2");
    bob_client.add_note_tag(pswap_tag).await?;
    bob_client.sync_state().await?;

    info!(note_id = %pswap_note.id(), account_id = %bob_account.id(), "Bob fully fills the PSWAP");
    let consume_request = TransactionRequestBuilder::new().build_pswap_consume(
        &pswap_note,
        bob_account.id(),
        REQUESTED_AMOUNT,
        0,
    )?;
    let payback_note_details = consume_request
        .expected_future_notes()
        .cloned()
        .map(|(n, _)| n)
        .collect::<Vec<_>>();
    assert_eq!(payback_note_details.len(), 1, "full fill should produce only the payback note");

    execute_tx_and_sync(&mut bob_client, bob_account.id(), consume_request).await?;

    // Alice consumes her payback note.
    alice_client.sync_state().await?;
    let payback_commitment = payback_note_details[0].commitment();
    let payback_note: Note = alice_client
        .get_input_note_by_commitment(payback_commitment)
        .await?
        .with_context(|| format!("Payback note {} not found", payback_commitment.to_hex()))?
        .try_into()?;
    let consume_payback =
        TransactionRequestBuilder::new().build_consume_notes(vec![payback_note])?;
    execute_tx_and_sync(&mut alice_client, alice_account.id(), consume_payback).await?;

    let alice_account_reader = alice_client.account_reader(alice_account.id());
    assert_eq!(
        alice_account_reader.get_balance(btc_faucet_account.id()).await?,
        MINT_AMOUNT - OFFERED_AMOUNT
    );
    assert_eq!(
        alice_account_reader.get_balance(eth_faucet_account.id()).await?,
        REQUESTED_AMOUNT
    );

    let bob_account_reader = bob_client.account_reader(bob_account.id());
    assert_eq!(bob_account_reader.get_balance(btc_faucet_account.id()).await?, OFFERED_AMOUNT);
    assert_eq!(
        bob_account_reader.get_balance(eth_faucet_account.id()).await?,
        MINT_AMOUNT - REQUESTED_AMOUNT
    );

    Ok(())
}

// PSWAP PARTIAL FILL ONCHAIN
// ================================================================================================

/// Verifies that partial fills produce the correct proportional payout and a remainder PSWAP note
/// with the right unfilled amounts.
pub async fn test_pswap_partial_fill_onchain(client_config: ClientConfig) -> Result<()> {
    const OFFERED_AMOUNT: u64 = 100;
    const REQUESTED_AMOUNT: u64 = 50;
    // Half fill: account_fill = 25, expected payout = 100 * 25 / 50 = 50.
    const ACCOUNT_FILL: u64 = 25;
    const EXPECTED_PAYOUT: u64 = 50;
    const REMAINING_OFFERED: u64 = OFFERED_AMOUNT - EXPECTED_PAYOUT;
    const REMAINING_REQUESTED: u64 = REQUESTED_AMOUNT - ACCOUNT_FILL;

    let (mut alice_client, alice_authenticator) = client_config.clone().into_client().await?;
    wait_for_node(&mut alice_client).await;
    let (mut bob_client, bob_authenticator) = ClientConfig::default()
        .with_rpc_endpoint(client_config.rpc_endpoint())
        .into_client()
        .await?;

    alice_client.sync_state().await?;
    bob_client.sync_state().await?;

    let (alice_account, ..) = insert_new_wallet(
        &mut alice_client,
        AccountType::Private,
        &alice_authenticator,
        RPO_FALCON_SCHEME_ID,
    )
    .await?;
    let (bob_account, ..) = insert_new_wallet(
        &mut bob_client,
        AccountType::Private,
        &bob_authenticator,
        RPO_FALCON_SCHEME_ID,
    )
    .await?;

    let (btc_faucet_account, _) = insert_new_fungible_faucet(
        &mut alice_client,
        AccountType::Private,
        &alice_authenticator,
        RPO_FALCON_SCHEME_ID,
    )
    .await?;
    let (eth_faucet_account, _) = insert_new_fungible_faucet(
        &mut bob_client,
        AccountType::Private,
        &bob_authenticator,
        RPO_FALCON_SCHEME_ID,
    )
    .await?;

    let tx_id = mint_and_consume(
        &mut alice_client,
        alice_account.id(),
        btc_faucet_account.id(),
        NoteType::Public,
    )
    .await;
    wait_for_tx(&mut alice_client, tx_id).await?;
    let tx_id = mint_and_consume(
        &mut bob_client,
        bob_account.id(),
        eth_faucet_account.id(),
        NoteType::Public,
    )
    .await;
    wait_for_tx(&mut bob_client, tx_id).await?;

    let offered_asset = FungibleAsset::new(btc_faucet_account.id(), OFFERED_AMOUNT)?;
    let requested_asset = FungibleAsset::new(eth_faucet_account.id(), REQUESTED_AMOUNT)?;

    let tx_request = TransactionRequestBuilder::new().build_pswap_create(
        &PswapTransactionData::new(alice_account.id(), offered_asset, requested_asset),
        NoteType::Public,
        NoteType::Public,
        None,
        alice_client.rng(),
    )?;
    let pswap_note = tx_request.expected_output_own_notes()[0].clone();
    execute_tx_and_sync(&mut alice_client, alice_account.id(), tx_request).await?;

    let pswap_tag = PswapNote::create_tag(NoteType::Public, &offered_asset, &requested_asset);
    bob_client.add_note_tag(pswap_tag).await?;
    bob_client.sync_state().await?;

    info!(account_fill = ACCOUNT_FILL, "Bob partially fills the PSWAP");
    let consume_request = TransactionRequestBuilder::new().build_pswap_consume(
        &pswap_note,
        bob_account.id(),
        ACCOUNT_FILL,
        0,
    )?;

    // The consume request should register both the payback p2id and the remainder PSWAP as
    // expected future notes.
    let future_notes: Vec<NoteDetails> =
        consume_request.expected_future_notes().cloned().map(|(n, _)| n).collect();
    assert_eq!(future_notes.len(), 2, "partial fill should produce a payback and a remainder");

    execute_tx_and_sync(&mut bob_client, bob_account.id(), consume_request).await?;

    // Bob spent only ACCOUNT_FILL of ETH and received EXPECTED_PAYOUT of BTC (proportional, not
    // the full offered amount). This is the assertion that catches a wrong NOTE_ARGS layout: a
    // wrong layout would either fall through to the script's full-fill default or be rejected.
    let bob_account_reader = bob_client.account_reader(bob_account.id());
    assert_eq!(
        bob_account_reader.get_balance(btc_faucet_account.id()).await?,
        EXPECTED_PAYOUT,
        "Bob should have received a proportional share, not the full offered amount"
    );
    assert_eq!(
        bob_account_reader.get_balance(eth_faucet_account.id()).await?,
        MINT_AMOUNT - ACCOUNT_FILL,
        "Bob should have spent only the partial fill amount"
    );

    // Locate the remainder PSWAP note among Bob's tracked input notes and verify its amounts.
    let commitments = future_notes.iter().map(|details| details.commitment()).collect();
    let bob_input_notes =
        bob_client.get_input_notes(NoteFilter::DetailsCommitments(commitments)).await?;
    let mut remainder_pswap = None;
    for record in &bob_input_notes {
        if let Ok(note) = TryInto::<Note>::try_into(record.clone())
            && let Ok(parsed) = PswapNote::try_from(&note)
        {
            remainder_pswap = Some(parsed);
            break;
        }
    }
    let remainder =
        remainder_pswap.context("remainder PSWAP note should exist after partial fill")?;
    assert_eq!(remainder.offered_asset().amount().as_u64(), REMAINING_OFFERED);
    assert_eq!(remainder.storage().requested_asset_amount(), REMAINING_REQUESTED);

    Ok(())
}

// PSWAP CANCEL ONCHAIN
// ================================================================================================

/// Verifies that the creator can cancel a PSWAP and reclaim the offered asset.
pub async fn test_pswap_cancel_onchain(client_config: ClientConfig) -> Result<()> {
    const OFFERED_AMOUNT: u64 = 100;
    const REQUESTED_AMOUNT: u64 = 50;

    let (mut alice_client, alice_authenticator) = client_config.into_client().await?;
    wait_for_node(&mut alice_client).await;
    alice_client.sync_state().await?;

    let (alice_account, ..) = insert_new_wallet(
        &mut alice_client,
        AccountType::Private,
        &alice_authenticator,
        RPO_FALCON_SCHEME_ID,
    )
    .await?;

    let (btc_faucet_account, _) = insert_new_fungible_faucet(
        &mut alice_client,
        AccountType::Private,
        &alice_authenticator,
        RPO_FALCON_SCHEME_ID,
    )
    .await?;
    // The requested-side faucet exists only so the FungibleAsset is well-formed.
    let (eth_faucet_account, _) = insert_new_fungible_faucet(
        &mut alice_client,
        AccountType::Private,
        &alice_authenticator,
        RPO_FALCON_SCHEME_ID,
    )
    .await?;

    let tx_id = mint_and_consume(
        &mut alice_client,
        alice_account.id(),
        btc_faucet_account.id(),
        NoteType::Private,
    )
    .await;
    wait_for_tx(&mut alice_client, tx_id).await?;

    let offered_asset = FungibleAsset::new(btc_faucet_account.id(), OFFERED_AMOUNT)?;
    let requested_asset = FungibleAsset::new(eth_faucet_account.id(), REQUESTED_AMOUNT)?;

    let create_request = TransactionRequestBuilder::new().build_pswap_create(
        &PswapTransactionData::new(alice_account.id(), offered_asset, requested_asset),
        NoteType::Private,
        NoteType::Private,
        None,
        alice_client.rng(),
    )?;
    let pswap_note = create_request.expected_output_own_notes()[0].clone();
    execute_tx_and_sync(&mut alice_client, alice_account.id(), create_request).await?;

    let alice_account_reader = alice_client.account_reader(alice_account.id());
    assert_eq!(
        alice_account_reader.get_balance(btc_faucet_account.id()).await?,
        MINT_AMOUNT - OFFERED_AMOUNT,
        "creating the PSWAP should debit the offered asset"
    );

    info!(note_id = %pswap_note.id(), "Alice cancels the PSWAP");
    let cancel_request =
        TransactionRequestBuilder::new().build_pswap_cancel(pswap_note, alice_account.id())?;
    execute_tx_and_sync(&mut alice_client, alice_account.id(), cancel_request).await?;

    let alice_account_reader = alice_client.account_reader(alice_account.id());
    assert_eq!(
        alice_account_reader.get_balance(btc_faucet_account.id()).await?,
        MINT_AMOUNT,
        "cancelling should restore the offered asset to the creator"
    );

    Ok(())
}
