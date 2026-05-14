use anyhow::{Context, Result};
use miden_client::Felt;
use miden_client::account::AccountStorageMode;
use miden_client::asset::{Asset, FungibleAsset};
use miden_client::auth::RPO_FALCON_SCHEME_ID;
use miden_client::note::NoteType;
use miden_client::store::TransactionFilter;
use miden_client::testing::common::*;
use miden_client::transaction::{
    PaymentNoteDescription,
    TransactionRequestBuilder,
    TransactionStatus,
};
use tracing::info;

use crate::tests::config::ClientConfig;

/// Real-node integration test for the `BatchBuilder` end-to-end path.
///
/// Mints tokens onto a first wallet, then submits two P2ID transfers from that
/// wallet to a second wallet as a single proven batch via
/// `Client::new_transaction_batch`.
///
/// The balance assertion at the end implicitly verifies `InMemoryBatchDataStore`'s
/// account state stacking: if the second push read the pre-batch state instead of
/// the post-push-1 state, both transactions would carry the same
/// `initial_account_state` in their proofs and the node would reject the batch.
/// Successful submission with balance = `MINT_AMOUNT` - 2 * `TRANSFER_AMOUNT` proves
/// that each push saw the state produced by the previous push.
pub async fn test_batch_builder_submits_two_p2id_on_one_account(
    client_config: ClientConfig,
) -> Result<()> {
    let (mut client, authenticator) = client_config.into_client().await?;
    wait_for_node(&mut client).await;

    let (first_regular_account, second_regular_account, faucet_account_header) =
        setup_two_wallets_and_faucet(
            &mut client,
            AccountStorageMode::Private,
            &authenticator,
            RPO_FALCON_SCHEME_ID,
        )
        .await?;

    let from_account_id = first_regular_account.id();
    let to_account_id = second_regular_account.id();
    let faucet_account_id = faucet_account_header.id();

    // Mint tokens into first_regular_account (covers both transfers).
    let tx_id =
        mint_and_consume(&mut client, from_account_id, faucet_account_id, NoteType::Private).await;
    wait_for_tx(&mut client, tx_id).await?;
    client.sync_state().await.unwrap();

    let nonce_before = client.account_reader(from_account_id).nonce().await?;
    info!(?nonce_before, "Sender nonce before batch");

    // Build two P2ID transfer requests of TRANSFER_AMOUNT each.
    let asset = FungibleAsset::new(faucet_account_id, TRANSFER_AMOUNT).unwrap();

    let tx_request_1 = TransactionRequestBuilder::new()
        .build_pay_to_id(
            PaymentNoteDescription::new(
                vec![Asset::Fungible(asset)],
                from_account_id,
                to_account_id,
            ),
            NoteType::Private,
            client.rng(),
        )
        .unwrap();

    let tx_request_2 = TransactionRequestBuilder::new()
        .build_pay_to_id(
            PaymentNoteDescription::new(
                vec![Asset::Fungible(asset)],
                from_account_id,
                to_account_id,
            ),
            NoteType::Private,
            client.rng(),
        )
        .unwrap();

    info!(
        from = %from_account_id,
        to = %to_account_id,
        amount = TRANSFER_AMOUNT,
        "Submitting 2-tx P2ID batch via BatchBuilder"
    );

    // Submit both requests as a single batch.
    let block_num = client
        .new_transaction_batch(from_account_id)
        .await?
        .push(tx_request_1)
        .await?
        .push(tx_request_2)
        .await?
        .submit()
        .await?;

    info!(block_num = block_num.as_u32(), "Batch submitted successfully");

    assert!(block_num.as_u32() > 0, "expected a positive block number from batch submit");

    // Poll until at least 3 sender-account transactions are committed (1 from
    // mint-and-consume + 2 from the batch). Give the node a reasonable window
    // to finalize the batch's block.
    let mut committed_count = 0;
    for attempt in 0..30 {
        wait_for_blocks(&mut client, 1).await;
        client.sync_state().await.unwrap();
        let all_transactions = client.get_transactions(TransactionFilter::All).await.unwrap();
        committed_count = all_transactions
            .iter()
            .filter(|tx| tx.details.account_id == from_account_id)
            .filter(|tx| matches!(tx.status, TransactionStatus::Committed { .. }))
            .count();
        info!(attempt, committed_count, "polling for batch txs to commit");
        if committed_count >= 3 {
            break;
        }
    }
    assert!(
        committed_count >= 3,
        "expected at least 3 committed transactions from the sender account \
         (1 mint-and-consume + 2 batch), got {committed_count}"
    );

    // Check that nonce has advanced by exactly 2.
    let nonce_after = client.account_reader(from_account_id).nonce().await?;
    info!(?nonce_before, ?nonce_after, "Sender nonce after batch");
    let expected = nonce_before + Felt::new(2);
    assert_eq!(
        nonce_after, expected,
        "sender nonce should advance by exactly 2 after a 2-tx batch \
         (stacking proof: {nonce_before:?} → {nonce_after:?}, expected {expected:?})"
    );

    // check that balance is handled correctly between batch txs
    let sender_balance = client
        .account_reader(from_account_id)
        .get_balance(faucet_account_id)
        .await
        .context("failed to find sender account after transactions")?;

    assert_eq!(
        sender_balance,
        MINT_AMOUNT - (TRANSFER_AMOUNT * 2),
        "sender balance should have decreased by exactly 2 * TRANSFER_AMOUNT — this proves \
         BatchBuilder stacked account state correctly between pushes"
    );

    Ok(())
}
