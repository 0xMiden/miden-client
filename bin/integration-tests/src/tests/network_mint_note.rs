use anyhow::{Context, Result, anyhow};
use miden_client::account::component::{
    AccountComponent,
    AccountComponentMetadata,
    NetworkFungibleFaucet,
    Ownable2Step,
    OwnerControlled,
    OwnerControlledInitConfig,
};
use miden_client::account::{AccountBuilder, AccountStorageMode, AccountType};
use miden_client::assembly::CodeBuilder;
use miden_client::asset::{FungibleAsset, TokenSymbol};
use miden_client::auth::RPO_FALCON_SCHEME_ID;
use miden_client::crypto::FeltRng;
use miden_client::note::{
    MintNote,
    MintNoteStorage,
    NetworkAccountTarget,
    NoteAssets,
    NoteAttachment,
    NoteExecutionHint,
    NoteFile,
    NoteId,
    NoteTag,
    NoteType,
    P2idNote,
    P2idNoteStorage,
};
use miden_client::store::InputNoteState;
use miden_client::testing::common::{
    execute_tx_and_sync,
    insert_new_wallet,
    wait_for_blocks,
    wait_for_tx,
};
use miden_client::transaction::TransactionRequestBuilder;
use miden_client::Felt;
use rand::RngCore;

const INCR_NONCE_AUTH_CODE: &str = "
    use miden::protocol::native_account

    @auth_script
    pub proc auth_basic
        exec.native_account::incr_nonce
        drop
    end
";

// Minimal no-op tx script: the faucet's `INCR_NONCE_AUTH_CODE` auth
// procedure already increments the nonce, so the script itself needs
// only to satisfy the builder's requirement that _some_ user code runs.
const NOOP_TX_SCRIPT: &str = "
    begin
        push.0 drop
    end
";

use crate::tests::config::ClientConfig;

/// End-to-end integration test for the standard MINT note → network faucet →
/// public P2ID output note flow.
///
/// Flow:
///   1. Alice (public wallet) first emits a zero-asset public P2ID note to Bob.
///      The node indexes the script of every committed public note, so this
///      registers `StandardNote::P2ID.script()` in the node's NTX script
///      registry. Without this, the node cannot materialize the P2ID output
///      note produced by the MINT note in step 2.
///   2. Alice owns a `NetworkFungibleFaucet` (network storage, no-auth,
///      Ownable2Step(alice)). She builds a `StandardNote::MINT` whose
///      `MintNoteStorage::new_public` encodes the P2ID recipient targeting
///      Bob and whose `NoteAttachment` is a `NetworkAccountTarget` pointing
///      at the faucet. The node's NTX builder consumes the MINT note against
///      the faucet; `mint_and_send` emits a public P2ID note carrying the
///      minted fungible asset to Bob.
///   3. Bob's client imports the expected P2ID `NoteId` and polls until it
///      reaches `InputNoteState::Committed`.
pub async fn test_ntx_mint_produces_public_p2id(client_config: ClientConfig) -> Result<()> {
    let (mut client, keystore) = client_config.clone().into_client().await?;
    let (mut client_2, keystore_2) = ClientConfig::default()
        .with_rpc_endpoint(client_config.rpc_endpoint())
        .into_client()
        .await?;

    let (alice, ..) = insert_new_wallet(
        &mut client,
        AccountStorageMode::Public,
        &keystore,
        RPO_FALCON_SCHEME_ID,
    )
    .await?;
    let (bob, ..) = insert_new_wallet(
        &mut client_2,
        AccountStorageMode::Public,
        &keystore_2,
        RPO_FALCON_SCHEME_ID,
    )
    .await?;

    // Deploy the network-storage fungible faucet owned by Alice. Minting is
    // gated note-side (the `mint_and_send` procedure checks that the MINT
    // note sender == the Ownable2Step owner), so the faucet only needs a
    // no-auth component that unconditionally increments its nonce — the
    // same pattern used by `deploy_counter_contract` for network-storage
    // accounts (see `bin/integration-tests/src/tests/network_transaction.rs`).
    let incr_nonce_auth_code = CodeBuilder::default()
        .compile_component_code("miden::testing::incr_nonce_auth", INCR_NONCE_AUTH_CODE)
        .context("failed to compile incr-nonce auth component")?;
    let incr_nonce_auth = AccountComponent::new(
        incr_nonce_auth_code,
        vec![],
        AccountComponentMetadata::new("miden::testing::incr_nonce_auth", AccountType::all()),
    )
    .map_err(|e| anyhow!("failed to create incr-nonce auth component: {e}"))?;

    let mut init_seed = [0u8; 32];
    client.rng().fill_bytes(&mut init_seed);
    let max_supply = Felt::new(9_999_999);
    let faucet = AccountBuilder::new(init_seed)
        .account_type(AccountType::FungibleFaucet)
        .storage_mode(AccountStorageMode::Network)
        .with_auth_component(incr_nonce_auth)
        .with_component(NetworkFungibleFaucet::new(
            TokenSymbol::new("MNT").unwrap(),
            10,
            max_supply,
        )?)
        .with_component(Ownable2Step::new(alice.id()))
        .with_component(OwnerControlled::new(OwnerControlledInitConfig::OwnerOnly))
        .build()
        .map_err(|e| anyhow!("failed to build network faucet: {e}"))?;
    client.add_account(&faucet, false).await?;

    // Commit the faucet's initial state on-chain via a trivial incr-nonce
    // tx submitted from the faucet itself; without this the node's NTX
    // builder has no knowledge of the faucet and cannot run the MINT note
    // against it.
    let deploy_script = CodeBuilder::new()
        .compile_tx_script(NOOP_TX_SCRIPT)
        .context("failed to compile faucet deploy tx script")?;
    let deploy_tx = TransactionRequestBuilder::new()
        .custom_script(deploy_script)
        .build()?;
    let deploy_tx_id = client.submit_new_transaction(faucet.id(), deploy_tx).await?;
    wait_for_tx(&mut client, deploy_tx_id).await?;

    // STEP 1: register the P2ID note script with the node by emitting a
    // zero-asset public P2ID note from Alice to Bob. Any committed public
    // note has its script indexed into the node's script registry — see
    // `TransactionRequestBuilder::build_register_note_scripts` docs for the
    // same mechanism used by `expected_ntx_scripts`.
    let p2id_pre = P2idNote::create(
        alice.id(),
        bob.id(),
        vec![],
        NoteType::Public,
        NoteAttachment::default(),
        client.rng(),
    )?;
    let register_tx = TransactionRequestBuilder::new()
        .own_output_notes(vec![p2id_pre])
        .build()?;
    execute_tx_and_sync(&mut client, alice.id(), register_tx).await?;

    // STEP 2: build the standard MINT note.
    // Precompute Bob's P2ID recipient + expected output NoteId.
    let amount = Felt::new(100);
    let serial_num = client.rng().draw_word();
    let bob_recipient = P2idNoteStorage::new(bob.id()).into_recipient(serial_num);
    let expected_asset = FungibleAsset::new(faucet.id(), amount.as_canonical_u64())?;
    let expected_output_id = NoteId::new(
        bob_recipient.digest(),
        NoteAssets::new(vec![expected_asset.into()])?.commitment(),
    );

    let mint_storage = MintNoteStorage::new_public(
        bob_recipient,
        amount,
        NoteTag::with_account_target(bob.id()).into(),
    )?;

    // The MINT note itself is routed to the network faucet via a
    // NetworkAccountTarget attachment.
    let target_ntx = NetworkAccountTarget::new(faucet.id(), NoteExecutionHint::Always)?;
    let mint_note = MintNote::create(
        faucet.id(),
        alice.id(), // must equal the faucet owner; checked by mint_and_send
        mint_storage,
        target_ntx.into(),
        client.rng(),
    )?;

    let mint_tx = TransactionRequestBuilder::new()
        .own_output_notes(vec![mint_note])
        .build()?;
    execute_tx_and_sync(&mut client, alice.id(), mint_tx).await?;

    // STEP 3: wait for the node's NTX builder to consume the MINT note and
    // emit the public P2ID; then observe it as Committed on Bob's client.
    for _ in 0..15 {
        wait_for_blocks(&mut client, 1).await;

        let _ = client_2
            .import_notes(&[NoteFile::NoteId(expected_output_id)])
            .await;
        client_2.sync_state().await?;
        if let Some(rec) = client_2.get_input_note(expected_output_id).await?
            && matches!(rec.state(), InputNoteState::Committed { .. })
        {
            return Ok(());
        }
    }

    Err(anyhow!(
        "timed out waiting for committed P2ID note {expected_output_id} emitted by network faucet"
    ))
}
