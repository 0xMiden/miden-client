//! Property-based test: two clients exchange assets over a 10k+ block chain with
//! note-relevant blocks scattered across large catch-up deltas.

use alloc::sync::Arc;
use std::env::temp_dir;

use miden_client::DebugMode;
use miden_client::account::Address;
use miden_client::auth::RPO_FALCON_SCHEME_ID;
use miden_client::builder::ClientBuilder;
use miden_client::keystore::FilesystemKeyStore;
use miden_client::store::{InputNoteState, NoteFilter};
use miden_client::testing::common::{
    MINT_AMOUNT,
    TRANSFER_AMOUNT,
    TestClient,
    create_test_store_path,
    insert_new_fungible_faucet,
    insert_new_wallet,
};
use miden_client::testing::mock::MockRpcApi;
use miden_client::transaction::{
    PaymentNoteDescription,
    TransactionRequestBuilder,
};
use miden_client_sqlite_store::ClientBuilderSqliteExt;
use miden_protocol::Felt;
use miden_protocol::account::{Account, AccountId, AccountStorageMode};
use miden_protocol::testing::account_id::ACCOUNT_ID_REGULAR_PRIVATE_ACCOUNT_UPDATABLE_CODE;
use miden_protocol::asset::FungibleAsset;
use miden_protocol::crypto::rand::RandomCoin;
use miden_protocol::note::NoteType;
use miden_testing::MockChain;
use miden_tx::LocalTransactionProver;
use proptest::prelude::*;
use rand::Rng;

// HELPERS
// ================================================================================================

async fn build_client(rpc: &MockRpcApi) -> anyhow::Result<(TestClient, FilesystemKeyStore)> {
    let seed: [u64; 4] = rand::rng().random();
    let keystore = FilesystemKeyStore::new(temp_dir())?;
    let client = ClientBuilder::new()
        .rpc(Arc::new(rpc.clone()))
        .rng(Box::new(RandomCoin::new(seed.map(Felt::new).into())))
        .sqlite_store(create_test_store_path())
        .authenticator(Arc::new(keystore.clone()))
        .in_debug_mode(DebugMode::Enabled)
        .tx_discard_delta(None)
        .build()
        .await?;
    Ok((client, keystore))
}

async fn execute_and_submit(
    client: &mut TestClient,
    account_id: AccountId,
    request: miden_client::transaction::TransactionRequest,
) -> anyhow::Result<()> {
    let tx = Box::pin(client.execute_transaction(account_id, request)).await?;
    let proven = LocalTransactionProver::default()
        .prove_dummy(tx.executed_transaction().clone())
        .map_err(|e| anyhow::anyhow!("prove_dummy failed: {e}"))?;
    let height = client.submit_proven_transaction(proven, &tx).await?;
    client.apply_transaction(&tx, height).await?;
    Ok(())
}

async fn mint(minter: &mut TestClient, faucet: &Account, target: AccountId) -> anyhow::Result<()> {
    let asset = FungibleAsset::new(faucet.id(), MINT_AMOUNT)?;
    let req = TransactionRequestBuilder::new()
        .build_mint_fungible_asset(asset, target, NoteType::Public, minter.rng())?;
    execute_and_submit(minter, faucet.id(), req).await
}

async fn transfer_p2id(
    client: &mut TestClient,
    sender: AccountId,
    recipient: AccountId,
    faucet_id: AccountId,
) -> anyhow::Result<()> {
    let asset = FungibleAsset::new(faucet_id, TRANSFER_AMOUNT)?.into();
    let req = TransactionRequestBuilder::new()
        .build_pay_to_id(
            PaymentNoteDescription::new(vec![asset], sender, recipient),
            NoteType::Public,
            client.rng(),
        )?;
    execute_and_submit(client, sender, req).await
}

async fn consume_all(client: &mut TestClient, wallet: AccountId) -> anyhow::Result<bool> {
    let committed: Vec<_> = client
        .get_input_notes(NoteFilter::Committed)
        .await?
        .iter()
        .filter(|n| matches!(n.state(), InputNoteState::Committed(_)))
        .filter_map(|n| n.try_into().ok())
        .collect();
    if committed.is_empty() {
        return Ok(false);
    }
    let req = TransactionRequestBuilder::new().build_consume_notes(committed)?;
    execute_and_submit(client, wallet, req).await?;
    Ok(true)
}

// ROUND
// ================================================================================================

#[derive(Debug, Clone)]
struct Round {
    mint_alice: bool,
    mint_bob: bool,
    /// Number of red herring notes to mint this round. These are notes whose tags
    /// match what the clients track but are addressed to a fake account nobody owns.
    /// Exercises the `found_relevant_note = false` path in state_sync.
    red_herrings: u32,
    /// Blocks between minting and the next sync. This is the gap that buries note
    /// blocks in the middle of a large delta.
    gap: u32,
    sync_alice: bool,
    sync_bob: bool,
}

fn round_strategy() -> impl Strategy<Value = Round> {
    (
        any::<bool>(),
        any::<bool>(),
        prop_oneof![
            3 => 0u32..=0,
            2 => 1u32..=3,
            1 => 4u32..=8,
        ],
        prop_oneof![
            3 => 0u32..=10,
            3 => 50u32..=200,
            2 => 300u32..=800,
            1 => 1000u32..=2000,
        ],
        any::<bool>(),
        any::<bool>(),
    )
        .prop_map(|(mint_alice, mint_bob, red_herrings, gap, sync_alice, sync_bob)| Round {
            mint_alice,
            mint_bob,
            red_herrings,
            gap,
            sync_alice,
            sync_bob,
        })
}

fn initial_gap_strategy() -> impl Strategy<Value = u32> {
    prop_oneof![
        2 => 248u32..=264,
        2 => 504u32..=520,
        2 => 1016u32..=1032,
        2 => 2040u32..=2056,
        1 => 4088u32..=4104,
        1 => 100u32..=500,
        1 => 600u32..=2000,
    ]
}

// CASE
// ================================================================================================

async fn run_case(initial_gap: u32, rounds: Vec<Round>) -> anyhow::Result<()> {
    let rpc = MockRpcApi::new(MockChain::new());
    let (mut alice, alice_ks) = build_client(&rpc).await?;
    let (mut bob, bob_ks) = build_client(&rpc).await?;

    let (faucet, _) = insert_new_fungible_faucet(
        &mut alice, AccountStorageMode::Public, &alice_ks, RPO_FALCON_SCHEME_ID,
    ).await?;
    let (alice_wallet, _) = insert_new_wallet(
        &mut alice, AccountStorageMode::Private, &alice_ks, RPO_FALCON_SCHEME_ID,
    ).await?;
    let (bob_wallet, _) = insert_new_wallet(
        &mut bob, AccountStorageMode::Private, &bob_ks, RPO_FALCON_SCHEME_ID,
    ).await?;

    let mut alice_minted: u64 = 0;
    let mut bob_minted: u64 = 0;

    // Red herring setup: a fake account that nobody owns. Both clients track its
    // tag. Notes minted to it appear in sync_notes (tag match) but the screener
    // discards them — exercising the `found_relevant_note = false` path.
    let fake_account_id =
        AccountId::try_from(ACCOUNT_ID_REGULAR_PRIVATE_ACCOUNT_UPDATABLE_CODE).unwrap();
    let fake_tag = Address::new(fake_account_id).to_note_tag();
    alice.add_note_tag(fake_tag).await?;
    bob.add_note_tag(fake_tag).await?;

    // Initial catch-up: large chain before any activity.
    rpc.advance_blocks(initial_gap);
    alice.sync_state().await?;
    bob.sync_state().await?;

    for round in &rounds {
        // Mint phase — creates note-relevant blocks.
        if round.mint_alice {
            mint(&mut alice, &faucet, alice_wallet.id()).await?;
            alice_minted += MINT_AMOUNT;
            rpc.advance_blocks(1);
        }
        if round.mint_bob {
            mint(&mut alice, &faucet, bob_wallet.id()).await?;
            bob_minted += MINT_AMOUNT;
            rpc.advance_blocks(1);
        }

        // Red herrings: mint to an account nobody owns. Both clients track its tag,
        // so sync_notes returns the blocks, but the screener discards the notes.
        for _ in 0..round.red_herrings {
            mint(&mut alice, &faucet, fake_account_id).await?;
            rpc.advance_blocks(1);
        }

        // Gap — buries the note blocks deep in the delta for whoever syncs next.
        if round.gap > 0 {
            rpc.advance_blocks(round.gap);
        }

        // Independent syncs — whichever client syncs here catches up across the gap.
        // The other accumulates lag for a future sync.
        if round.sync_alice {
            alice.sync_state().await?;
        }
        if round.sync_bob {
            bob.sync_state().await?;
        }

        // Consume + transfer: best-effort. These generate more on-chain activity
        // (and more note-relevant blocks) but failures here aren't sync bugs.
        consume_all(&mut alice, alice_wallet.id()).await.ok();
        consume_all(&mut bob, bob_wallet.id()).await.ok();
        rpc.advance_blocks(1);

        // Attempt a transfer from whoever has balance.
        if alice_minted >= TRANSFER_AMOUNT {
            alice.sync_state().await?;
            transfer_p2id(&mut alice, alice_wallet.id(), bob_wallet.id(), faucet.id()).await.ok();
            rpc.advance_blocks(1);
        }
    }

    // Final: sync everything, consume everything.
    alice.sync_state().await?;
    bob.sync_state().await?;
    consume_all(&mut alice, alice_wallet.id()).await.ok();
    consume_all(&mut bob, bob_wallet.id()).await.ok();
    rpc.advance_blocks(1);
    alice.sync_state().await?;
    bob.sync_state().await?;

    // Invariant: total supply across both wallets must equal total minted. Transfers
    // just move assets between them so they cancel out.
    let actual_alice = alice.account_reader(alice_wallet.id()).get_balance(faucet.id()).await?;
    let actual_bob = bob.account_reader(bob_wallet.id()).get_balance(faucet.id()).await?;
    let total_minted = alice_minted + bob_minted;

    anyhow::ensure!(
        actual_alice + actual_bob <= total_minted,
        "total balance ({} + {} = {}) exceeds total minted ({total_minted})",
        actual_alice,
        actual_bob,
        actual_alice + actual_bob,
    );

    Ok(())
}

// PROPTEST
// ================================================================================================

const NUM_CASES: u32 = 64;

proptest! {
    #![proptest_config(ProptestConfig {
        cases: NUM_CASES,
        max_shrink_iters: 64,
        .. ProptestConfig::default()
    })]

    #[test]
    fn two_clients_exchanging_assets_over_long_chain(
        initial_gap in initial_gap_strategy(),
        rounds in prop::collection::vec(round_strategy(), 8..=25),
    ) {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("failed to build tokio runtime");

        rt.block_on(run_case(initial_gap, rounds))
            .map_err(|e| TestCaseError::fail(format!("case failed: {e:#}")))?;
    }
}
