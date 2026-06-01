use std::sync::{Arc, LazyLock};
use std::vec;

use anyhow::{Context, Result, anyhow};
use miden_client::account::component::{
    AccountComponent,
    AccountComponentMetadata,
    BurnPolicyConfig,
    FungibleFaucet,
    MintPolicyConfig,
    Ownable2Step,
    PolicyAuthority,
    TokenName,
    TokenPolicyManager,
};
use miden_client::account::{
    Account,
    AccountBuilder,
    AccountBuilderSchemaCommitmentExt,
    AccountId,
    AccountStorageMode,
    AccountType,
    StorageSlot,
    StorageSlotName,
};
use miden_client::assembly::{CodeBuilder, Library, Module, ModuleKind, Path, SourceManagerSync};
use miden_client::asset::{AssetAmount, FungibleAsset, TokenSymbol};
use miden_client::auth::RPO_FALCON_SCHEME_ID;
use miden_client::crypto::FeltRng;
use miden_client::note::{
    MintNote,
    MintNoteStorage,
    NetworkAccountTarget,
    Note,
    NoteAssets,
    NoteAttachment,
    NoteAttachments,
    NoteExecutionHint,
    NoteFile,
    NoteId,
    NoteRecipient,
    NoteScript,
    NoteStorage,
    NoteTag,
    NoteType,
    P2idNoteStorage,
    PartialNoteMetadata,
    StandardNote,
};
use miden_client::store::InputNoteState;
use miden_client::sync::NoteTagSource;
use miden_client::testing::common::{
    TestClient,
    assert_account_has_single_asset,
    consume_notes,
    execute_tx_and_sync,
    insert_new_wallet,
    wait_for_blocks,
    wait_for_tx,
};
use miden_client::transaction::{TransactionKernel, TransactionRequestBuilder};
use miden_client::{Felt, Word, ZERO};
use rand::{Rng, RngCore};

use crate::tests::config::ClientConfig;

// HELPERS
// ================================================================================================

pub(crate) static COUNTER_SLOT_NAME: LazyLock<StorageSlotName> = LazyLock::new(|| {
    StorageSlotName::new("miden::testing::counter_contract::counter").expect("slot name is valid")
});

const COUNTER_CONTRACT: &str = r#"
        use miden::protocol::active_account
        use miden::protocol::native_account
        use miden::core::word
        use miden::core::sys

        const COUNTER_SLOT = word("miden::testing::counter_contract::counter")

        # => []
        pub proc get_count
            push.COUNTER_SLOT[0..2] exec.active_account::get_item
            exec.sys::truncate_stack
        end

        # => []
        pub proc increment_count
            push.COUNTER_SLOT[0..2] exec.active_account::get_item
            # => [count]
            push.1 add
            # => [count+1]
            push.COUNTER_SLOT[0..2] exec.native_account::set_item
            # => []
            exec.sys::truncate_stack
            # => []
        end"#;

const INCR_NONCE_AUTH_CODE: &str = "
    use miden::protocol::native_account

    @auth_script
    pub proc auth_basic
        exec.native_account::incr_nonce
        drop
    end
";

const INCR_SCRIPT_CODE: &str = "
    use external_contract::counter_contract
    begin
        call.counter_contract::increment_count
    end
";

const INCR_NOTE_SCRIPT_CODE: &str = "
    use external_contract::counter_contract
    @note_script
    pub proc main
        call.counter_contract::increment_count
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

// A non-standard "claim to target" note script: it asserts the consuming account is the note's
// target (read from the note's storage) and then moves all of the note's assets into that
// account's vault. It is functionally similar to P2ID but hand-written, so its MAST root differs
// from every standard note script — exactly the case the node's NTX builder cannot resolve without
// the script being pre-registered. The `{nonce}` placeholder is replaced with a per-test value so
// the compiled root is unique per run and can never collide with a previously registered script on
// a shared node.
const NON_STANDARD_CLAIM_NOTE_SCRIPT: &str = r#"
    use miden::protocol::active_account
    use miden::protocol::account_id
    use miden::protocol::active_note
    use miden::standards::wallets::basic->basic_wallet

    @note_script
    pub proc main
        # drop the note arguments
        dropw

        # mix in a per-test nonce so this script's MAST root is unique per run
        push.{nonce} drop

        # load the note storage into memory starting at address 0
        push.0 exec.active_note::get_storage
        # => [num_storage_items, storage_ptr]

        # this script expects exactly the 2 storage items of an account id (suffix, prefix)
        eq.2 assert.err="non-standard claim note expects exactly 2 storage items"
        # => [storage_ptr]

        # read the target account id (suffix, prefix) from the note storage
        dup add.1 mem_load swap mem_load
        # => [target_account_id_suffix, target_account_id_prefix]

        exec.active_account::get_id
        # => [account_id_suffix, account_id_prefix, target_account_id_suffix, target_account_id_prefix]

        exec.account_id::is_equal assert.err="consumer is not the note's target account"
        # => []

        # move all of the note's assets into the consuming account's vault
        exec.basic_wallet::add_assets_to_account
        # => []
    end
"#;

/// Deploys a counter contract as a network account
pub(crate) async fn deploy_counter_contract(
    client: &mut TestClient,
    storage_mode: AccountStorageMode,
) -> Result<Account> {
    let acc = get_counter_contract_account(client, storage_mode).await?;

    client.add_account(&acc, false).await?;

    let source_manager = client.source_manager();
    let mut script_builder = CodeBuilder::with_source_manager(source_manager.clone());
    script_builder.link_dynamic_library(&counter_contract_library(source_manager))?;
    let tx_script = script_builder.compile_tx_script(INCR_SCRIPT_CODE)?;

    // Build a transaction request with the custom script
    let tx_increment_request = TransactionRequestBuilder::new().custom_script(tx_script).build()?;

    // Execute the transaction locally
    let tx_id = client.submit_new_transaction(acc.id(), tx_increment_request).await?;
    wait_for_tx(client, tx_id).await?;

    Ok(acc)
}

async fn get_counter_contract_account(
    client: &mut TestClient,
    storage_mode: AccountStorageMode,
) -> Result<Account> {
    let counter_slot = StorageSlot::with_empty_value(COUNTER_SLOT_NAME.clone());
    let counter_code = CodeBuilder::default()
        .compile_component_code("miden::testing::counter_contract", COUNTER_CONTRACT)
        .context("failed to compile counter contract component code")?;
    let counter_component = AccountComponent::new(
        counter_code,
        vec![counter_slot],
        AccountComponentMetadata::new("miden::testing::counter_component", AccountType::all()),
    )
    .map_err(|err| anyhow::anyhow!(err))
    .context("failed to create counter contract component")?;

    let incr_nonce_auth_code = CodeBuilder::default()
        .compile_component_code("miden::testing::incr_nonce_auth", INCR_NONCE_AUTH_CODE)
        .context("failed to compile increment nonce auth component code")?;
    let incr_nonce_auth = AccountComponent::new(
        incr_nonce_auth_code,
        vec![],
        AccountComponentMetadata::new("miden::testing::incr_nonce_auth", AccountType::all()),
    )
    .map_err(|err| anyhow::anyhow!(err))
    .context("failed to create increment nonce auth component")?;

    let mut init_seed = [0u8; 32];
    client.rng().fill_bytes(&mut init_seed);

    let account = AccountBuilder::new(init_seed)
        .storage_mode(storage_mode)
        .with_component(counter_component)
        .with_auth_component(incr_nonce_auth)
        .build_with_schema_commitment()
        .context("failed to build account with counter contract")?;

    Ok(account)
}

/// Deploys a network-storage fungible faucet owned by `owner_id` and commits its initial state
/// on-chain.
///
/// Minting is gated note-side (the `mint_and_send` procedure checks that the MINT note sender ==
/// the `Ownable2Step` owner), so the faucet only needs a no-auth component that unconditionally
/// increments its nonce — the same pattern `deploy_counter_contract` uses for network accounts.
/// The trailing no-op tx is required so the node's NTX builder knows about the faucet before any
/// MINT note runs against it.
async fn deploy_network_fungible_faucet(
    client: &mut TestClient,
    owner_id: AccountId,
) -> Result<Account> {
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
    let symbol = TokenSymbol::new("MNT").unwrap();
    let name = TokenName::new(&symbol.to_string()).expect("token symbol is a valid token name");
    let max_supply: u64 = 9_999_999;
    let fungible_faucet = FungibleFaucet::builder()
        .name(name)
        .symbol(symbol)
        .decimals(10)
        .max_supply(AssetAmount::new(max_supply).map_err(|e| anyhow!("invalid max supply: {e}"))?)
        .build()
        .map_err(|e| anyhow!("failed to build fungible faucet: {e}"))?;
    let faucet = AccountBuilder::new(init_seed)
        .account_type(AccountType::FungibleFaucet)
        .storage_mode(AccountStorageMode::Network)
        .with_auth_component(incr_nonce_auth)
        .with_component(fungible_faucet)
        .with_components(TokenPolicyManager::new(
            PolicyAuthority::OwnerControlled,
            MintPolicyConfig::OwnerOnly,
            BurnPolicyConfig::AllowAll,
        ))
        .with_component(Ownable2Step::new(owner_id))
        .build_with_schema_commitment()
        .map_err(|e| anyhow!("failed to build network faucet: {e}"))?;
    client.add_account(&faucet, false).await?;

    let deploy_script = CodeBuilder::with_source_manager(client.source_manager())
        .compile_tx_script(NOOP_TX_SCRIPT)
        .context("failed to compile faucet deploy tx script")?;
    let deploy_tx = TransactionRequestBuilder::new().custom_script(deploy_script).build()?;
    let deploy_tx_id = client.submit_new_transaction(faucet.id(), deploy_tx).await?;
    wait_for_tx(client, deploy_tx_id).await?;

    Ok(faucet)
}

/// Compiles the [`NON_STANDARD_CLAIM_NOTE_SCRIPT`] with a unique `nonce` and returns it together
/// with the note storage encoding `target`'s account id (the script asserts the consumer matches
/// it). The compiled script is guaranteed non-standard; callers assert that before relying on the
/// script-registration path.
fn build_non_standard_claim_note(
    client: &TestClient,
    target: AccountId,
    nonce: u32,
) -> Result<(NoteScript, NoteStorage)> {
    let script_src = NON_STANDARD_CLAIM_NOTE_SCRIPT.replace("{nonce}", &nonce.to_string());
    let script = client
        .code_builder()
        .compile_note_script(script_src.as_str())
        .context("failed to compile non-standard claim note script")?;
    let storage = NoteStorage::new(vec![target.suffix(), target.prefix().as_felt()])
        .context("failed to build non-standard claim note storage")?;
    Ok((script, storage))
}

// TESTS
// ================================================================================================

pub async fn test_counter_contract_ntx(client_config: ClientConfig) -> Result<()> {
    const BUMP_NOTE_NUMBER: u64 = 5;
    let (mut client, keystore) = client_config.into_client().await?;
    client.sync_state().await?;

    let network_account = deploy_counter_contract(&mut client, AccountStorageMode::Network).await?;

    let counter_value = client
        .account_reader(network_account.id())
        .get_storage_item(COUNTER_SLOT_NAME.clone())
        .await
        .context("failed to find network account after deployment")?;
    assert_eq!(counter_value, Word::from([Felt::new(1), ZERO, ZERO, ZERO]));

    let (native_account, ..) =
        insert_new_wallet(&mut client, AccountStorageMode::Public, &keystore, RPO_FALCON_SCHEME_ID)
            .await?;

    let mut network_notes = vec![];

    let source_manager = client.source_manager();
    for _ in 0..BUMP_NOTE_NUMBER {
        let network_note = get_network_note(
            native_account.id(),
            network_account.id(),
            source_manager.clone(),
            &mut client.rng(),
        )?;
        network_notes.push(network_note);
    }

    let tx_request = TransactionRequestBuilder::new().own_output_notes(network_notes).build()?;

    execute_tx_and_sync(&mut client, native_account.id(), tx_request).await?;

    // Wait for the node to consume the network notes in subsequent blocks
    let expected_counter = Word::from([Felt::new(1 + BUMP_NOTE_NUMBER), ZERO, ZERO, ZERO]);
    for _ in 0..10 {
        let a = client
            .test_rpc_api()
            .get_account_details(network_account.id())
            .await?
            .account()
            .cloned()
            .with_context(|| "account details not available")?;

        if a.storage().get_item(&COUNTER_SLOT_NAME)? == expected_counter {
            return Ok(());
        }

        wait_for_blocks(&mut client, 1).await;
    }

    let a = client
        .test_rpc_api()
        .get_account_details(network_account.id())
        .await?
        .account()
        .cloned()
        .with_context(|| "account details not available")?;

    assert_eq!(a.storage().get_item(&COUNTER_SLOT_NAME)?, expected_counter);
    Ok(())
}

pub async fn test_recall_note_before_ntx_consumes_it(client_config: ClientConfig) -> Result<()> {
    let (mut client, keystore) = client_config.into_client().await?;
    client.sync_state().await?;

    let network_account = deploy_counter_contract(&mut client, AccountStorageMode::Network).await?;
    let native_account = deploy_counter_contract(&mut client, AccountStorageMode::Public).await?;

    let wallet =
        insert_new_wallet(&mut client, AccountStorageMode::Public, &keystore, RPO_FALCON_SCHEME_ID)
            .await?
            .0;

    let network_note = get_network_note(
        wallet.id(),
        network_account.id(),
        client.source_manager(),
        &mut client.rng(),
    )?;
    // Prepare both transactions
    let tx_request = TransactionRequestBuilder::new()
        .own_output_notes(vec![network_note.clone()])
        .build()?;

    let bump_result = client.execute_transaction(wallet.id(), tx_request).await?;
    let current_height = client.get_sync_height().await?;
    client.apply_transaction(&bump_result, current_height).await?;

    let tx_request = TransactionRequestBuilder::new()
        .input_notes(vec![(network_note, None)])
        .build()?;

    let consume_result = client.execute_transaction(native_account.id(), tx_request).await?;
    let bump_proven = client.prove_transaction(&bump_result).await?;
    let consume_proven = client.prove_transaction(&consume_result).await?;

    // Submit both transactions
    let _bump_submission_height =
        client.submit_proven_transaction(bump_proven, &bump_result).await?;

    let consume_submission_height =
        client.submit_proven_transaction(consume_proven, &consume_result).await?;
    client.apply_transaction(&consume_result, consume_submission_height).await?;

    wait_for_blocks(&mut client, 2).await;

    // The network account should have original value
    let network_counter = client
        .account_reader(network_account.id())
        .get_storage_item(COUNTER_SLOT_NAME.clone())
        .await
        .context("failed to find network account after recall test")?;
    assert_eq!(network_counter, Word::from([Felt::new(1), ZERO, ZERO, ZERO]));

    // The native account should have the incremented value
    let native_counter = client
        .account_reader(native_account.id())
        .get_storage_item(COUNTER_SLOT_NAME.clone())
        .await
        .context("failed to find native account after recall test")?;
    assert_eq!(native_counter, Word::from([Felt::new(2), ZERO, ZERO, ZERO]));
    Ok(())
}

/// Validates end-to-end against a real node that a note created for a network account is consumed
/// by that account and the client attributes the consumption to it.
///
/// The network account consumes the note via same-batch erasure, whose RPC stream carries only the
/// `NoteHeader`. The network-account target lives in the note attachment (not delivered by that
/// stream), but the creating client holds the attachments locally on the output note, so it derives
/// the consumer and records the note as consumed by the network account.
pub async fn test_network_note_consumed_by_ntx(client_config: ClientConfig) -> Result<()> {
    let (mut client, keystore) = client_config.into_client().await?;
    client.sync_state().await?;

    let network_account = deploy_counter_contract(&mut client, AccountStorageMode::Network).await?;
    let network_account_id = network_account.id();

    let (sender_account, ..) =
        insert_new_wallet(&mut client, AccountStorageMode::Public, &keystore, RPO_FALCON_SCHEME_ID)
            .await?;

    let network_note = get_network_note(
        sender_account.id(),
        network_account_id,
        client.source_manager(),
        &mut client.rng(),
    )?;
    let note_id = network_note.id();

    let tx_request =
        TransactionRequestBuilder::new().own_output_notes(vec![network_note]).build()?;
    execute_tx_and_sync(&mut client, sender_account.id(), tx_request).await?;

    // Wait for the network account to consume the note (check counter increment).
    let expected_counter = Word::from([Felt::new(2), ZERO, ZERO, ZERO]);
    for _ in 0..15 {
        client.sync_state().await?;
        let account_details = client
            .test_rpc_api()
            .get_account_details(network_account_id)
            .await?
            .account()
            .cloned()
            .with_context(|| "account details not available")?;

        if account_details.storage().get_item(&COUNTER_SLOT_NAME)? == expected_counter {
            break;
        }
        wait_for_blocks(&mut client, 1).await;
    }

    // The note is consumed via same-batch erasure. The creating client derives the consumer from
    // the output note's attachments, so it records the note as consumed by the network account.
    // Poll until the client records the consumption.
    let mut consumer = None;
    for _ in 0..10 {
        client.sync_state().await?;
        if let Some(record) = client.get_input_note(note_id).await?
            && record.is_consumed()
        {
            consumer = record.consumer_account();
            break;
        }
        wait_for_blocks(&mut client, 1).await;
    }

    assert_eq!(
        consumer,
        Some(network_account_id),
        "the network note's consumer should be the network account"
    );

    Ok(())
}

/// End-to-end integration test for the standard MINT note → network faucet →
/// public P2ID output note flow.
///
/// The output is a standard P2ID note, which the node's NTX builder resolves directly, so no
/// script pre-registration is needed.
///
/// Flow:
///   1. Alice owns a `NetworkFungibleFaucet` (network storage, no-auth, Ownable2Step(alice)). She
///      builds a `StandardNote::MINT` whose `MintNoteStorage::new_public` encodes the P2ID
///      recipient targeting Bob and whose `NoteAttachment` is a `NetworkAccountTarget` pointing at
///      the faucet. The node's NTX builder consumes the MINT note against the faucet;
///      `mint_and_send` emits a public P2ID note carrying the minted fungible asset to Bob.
///   2. Bob's client imports the expected P2ID `NoteId` and polls until it reaches
///      `InputNoteState::Committed`.
pub async fn test_ntx_mint_produces_public_p2id(client_config: ClientConfig) -> Result<()> {
    let (mut client, keystore) = client_config.clone().into_client().await?;
    let (mut client_2, keystore_2) = ClientConfig::default()
        .with_rpc_endpoint(client_config.rpc_endpoint())
        .into_client()
        .await?;

    let (alice, ..) =
        insert_new_wallet(&mut client, AccountStorageMode::Public, &keystore, RPO_FALCON_SCHEME_ID)
            .await?;
    let (bob, ..) = insert_new_wallet(
        &mut client_2,
        AccountStorageMode::Public,
        &keystore_2,
        RPO_FALCON_SCHEME_ID,
    )
    .await?;

    // Deploy the network-storage fungible faucet owned by Alice.
    let faucet = deploy_network_fungible_faucet(&mut client, alice.id()).await?;

    // Build the standard MINT note. Precompute Bob's P2ID recipient + expected
    // output NoteId so we can poll for it on client_2.
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
    let attachments = NoteAttachments::new(vec![target_ntx.into()])?;
    let mint_note = MintNote::create(
        faucet.id(),
        alice.id(), // must equal the faucet owner; checked by mint_and_send
        mint_storage,
        attachments,
        client.rng(),
    )?;

    let mint_tx = TransactionRequestBuilder::new().own_output_notes(vec![mint_note]).build()?;
    execute_tx_and_sync(&mut client, alice.id(), mint_tx).await?;

    // Wait for the node's NTX builder to consume the MINT note and emit the
    // public P2ID; then observe it as Committed on Bob's client.
    for _ in 0..15 {
        wait_for_blocks(&mut client, 1).await;

        let _ = client_2.import_notes(&[NoteFile::NoteId(expected_output_id)]).await;
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

/// End-to-end NTX integration test for a public output note carrying a NON-STANDARD note script.
///
/// This is the general case of [`test_ntx_mint_produces_public_p2id`]. There the output is a
/// standard P2ID note, which the node's NTX builder resolves directly, so no script
/// pre-registration is needed. Here the output note uses a custom (non-standard) script, which the
/// NTX builder must resolve from its registry when it builds the public output note, so the script
/// has to be registered on the node before the NTX runs.
///
/// Flow:
///   1. Alice owns a network `NetworkFungibleFaucet`. The MINT note's public output recipient uses
///      a custom "claim to target" note script (asserts the consumer is Bob, then moves the minted
///      asset into Bob's vault). Its MAST root is unique per run, so it is neither a standard
///      script nor a previously registered one.
///   2. Alice pre-registers the script via [`TransactionRequestBuilder::expected_ntx_scripts`] on a
///      trivial no-op transaction, then waits for that registration to be committed on-chain. The
///      NTX builder resolves the output note's script while it executes, so the script must be
///      committed before the MINT runs. `expected_ntx_scripts` submits the registration note but
///      does not itself wait for it to commit, so it is set on an up-front no-op tx (not the MINT
///      request) and followed by an explicit wait — otherwise the NTX could run before the script
///      lands.
///   3. Alice mints. The node's NTX builder consumes the MINT note and emits the public note with
///      the custom script. Bob's client imports the expected `NoteId`, observes it `Committed`,
///      consumes it, and ends up holding the minted asset.
pub async fn test_ntx_mint_produces_public_note_with_non_standard_script(
    client_config: ClientConfig,
) -> Result<()> {
    let (mut client, keystore) = client_config.clone().into_client().await?;
    let (mut client_2, keystore_2) = ClientConfig::default()
        .with_rpc_endpoint(client_config.rpc_endpoint())
        .into_client()
        .await?;

    let (alice, ..) =
        insert_new_wallet(&mut client, AccountStorageMode::Public, &keystore, RPO_FALCON_SCHEME_ID)
            .await?;
    let (bob, ..) = insert_new_wallet(
        &mut client_2,
        AccountStorageMode::Public,
        &keystore_2,
        RPO_FALCON_SCHEME_ID,
    )
    .await?;

    let faucet = deploy_network_fungible_faucet(&mut client, alice.id()).await?;

    // Build the custom (non-standard) output recipient targeting Bob.
    let amount = Felt::new(100);
    let serial_num = client.rng().draw_word();
    let nonce: u32 = client.rng().random();
    let (custom_script, custom_storage) = build_non_standard_claim_note(&client, bob.id(), nonce)?;
    assert!(
        StandardNote::from_script(&custom_script).is_none(),
        "the claim script must be non-standard for this test to exercise script pre-registration",
    );
    let recipient = NoteRecipient::new(serial_num, custom_script.clone(), custom_storage);

    let expected_asset = FungibleAsset::new(faucet.id(), amount.as_canonical_u64())?;
    let expected_output_id =
        NoteId::new(recipient.digest(), NoteAssets::new(vec![expected_asset.into()])?.commitment());

    // Build the MINT note routed to the network faucet, carrying the custom public output
    // recipient.
    let mint_storage = MintNoteStorage::new_public(
        recipient,
        amount,
        NoteTag::with_account_target(bob.id()).into(),
    )?;
    let target_ntx = NetworkAccountTarget::new(faucet.id(), NoteExecutionHint::Always)?;
    let attachments = NoteAttachments::new(vec![target_ntx.into()])?;
    let mint_note = MintNote::create(
        faucet.id(),
        alice.id(), // must equal the faucet owner; checked by mint_and_send
        mint_storage,
        attachments,
        client.rng(),
    )?;

    // Pre-register the non-standard output script via `expected_ntx_scripts`, then wait for the
    // registration to commit before minting. Setting `expected_ntx_scripts` makes the client submit
    // a public note carrying the script before the request's own transaction runs, so it is
    // attached to a trivial no-op tx that runs up front. The NTX builder resolves the output note's
    // script while it executes, so the registration must be committed (and indexed) before the MINT
    // runs — `execute_tx_and_sync` waits for the no-op tx (committed alongside the registration
    // note), and the extra block adds an indexing margin. Setting `expected_ntx_scripts` on the
    // MINT request itself would race: the registration would not be committed before the NTX
    // runs.
    let noop_script = client
        .code_builder()
        .compile_tx_script(NOOP_TX_SCRIPT)
        .context("failed to compile no-op registration tx script")?;
    let register_tx = TransactionRequestBuilder::new()
        .custom_script(noop_script)
        .expected_ntx_scripts(vec![custom_script])
        .build()?;
    execute_tx_and_sync(&mut client, alice.id(), register_tx).await?;
    wait_for_blocks(&mut client, 1).await;

    let mint_tx = TransactionRequestBuilder::new().own_output_notes(vec![mint_note]).build()?;
    execute_tx_and_sync(&mut client, alice.id(), mint_tx).await?;

    // Wait for the NTX builder to emit the public note; observe it `Committed` on Bob's client.
    let mut committed = false;
    for _ in 0..15 {
        wait_for_blocks(&mut client, 1).await;

        let _ = client_2.import_notes(&[NoteFile::NoteId(expected_output_id)]).await;
        client_2.sync_state().await?;
        if let Some(rec) = client_2.get_input_note(expected_output_id).await?
            && matches!(rec.state(), InputNoteState::Committed { .. })
        {
            committed = true;
            break;
        }
    }
    if !committed {
        return Err(anyhow!(
            "timed out waiting for committed public note {expected_output_id} with a non-standard script"
        ));
    }

    // Bob consumes the public note; the custom script moves the minted asset into his vault.
    let note: Note = client_2
        .get_input_note(expected_output_id)
        .await?
        .context("expected the committed public note to be present on Bob's client")?
        .try_into()?;
    let consume_tx_id = consume_notes(&mut client_2, bob.id(), &[note]).await;
    wait_for_tx(&mut client_2, consume_tx_id).await?;

    assert_account_has_single_asset(&client_2, bob.id(), faucet.id(), amount.as_canonical_u64())
        .await;
    Ok(())
}

/// Negative counterpart to [`test_ntx_mint_produces_public_note_with_non_standard_script`].
///
/// When a MINT note's public output recipient uses a non-standard script that was NOT
/// pre-registered, the node's NTX builder cannot reconstruct the output note (its script is missing
/// from the registry), so the network transaction silently fails to produce the note. This test
/// omits any registration and asserts the expected public note never reaches `Committed` on the
/// observer client.
///
/// The check is inherently timeout-based: we wait a bounded number of blocks and confirm the note
/// never appears. The script's MAST root is randomized per run so it can never have been registered
/// by an earlier test run against a shared node.
pub async fn test_ntx_non_standard_script_without_registration_is_not_created(
    client_config: ClientConfig,
) -> Result<()> {
    let (mut client, keystore) = client_config.clone().into_client().await?;
    let (mut client_2, keystore_2) = ClientConfig::default()
        .with_rpc_endpoint(client_config.rpc_endpoint())
        .into_client()
        .await?;

    let (alice, ..) =
        insert_new_wallet(&mut client, AccountStorageMode::Public, &keystore, RPO_FALCON_SCHEME_ID)
            .await?;
    let (bob, ..) = insert_new_wallet(
        &mut client_2,
        AccountStorageMode::Public,
        &keystore_2,
        RPO_FALCON_SCHEME_ID,
    )
    .await?;

    let faucet = deploy_network_fungible_faucet(&mut client, alice.id()).await?;

    let amount = Felt::new(100);
    let serial_num = client.rng().draw_word();
    let nonce: u32 = client.rng().random();
    let (custom_script, custom_storage) = build_non_standard_claim_note(&client, bob.id(), nonce)?;
    assert!(
        StandardNote::from_script(&custom_script).is_none(),
        "the claim script must be non-standard for this test to be meaningful",
    );
    let recipient = NoteRecipient::new(serial_num, custom_script, custom_storage);

    let expected_asset = FungibleAsset::new(faucet.id(), amount.as_canonical_u64())?;
    let expected_output_id =
        NoteId::new(recipient.digest(), NoteAssets::new(vec![expected_asset.into()])?.commitment());

    let mint_storage = MintNoteStorage::new_public(
        recipient,
        amount,
        NoteTag::with_account_target(bob.id()).into(),
    )?;
    let target_ntx = NetworkAccountTarget::new(faucet.id(), NoteExecutionHint::Always)?;
    let attachments = NoteAttachments::new(vec![target_ntx.into()])?;
    let mint_note =
        MintNote::create(faucet.id(), alice.id(), mint_storage, attachments, client.rng())?;

    // Note: the custom script is never registered on the node.
    let mint_tx = TransactionRequestBuilder::new().own_output_notes(vec![mint_note]).build()?;
    execute_tx_and_sync(&mut client, alice.id(), mint_tx).await?;

    // Give the NTX builder ample time; the public note must never become `Committed`.
    for _ in 0..10 {
        wait_for_blocks(&mut client, 1).await;
        let _ = client_2.import_notes(&[NoteFile::NoteId(expected_output_id)]).await;
        client_2.sync_state().await?;
        if let Some(rec) = client_2.get_input_note(expected_output_id).await? {
            assert!(
                !matches!(rec.state(), InputNoteState::Committed { .. }),
                "NTX must not produce a committed public note for an unregistered non-standard script",
            );
        }
    }

    let is_committed = client_2
        .get_input_note(expected_output_id)
        .await?
        .as_ref()
        .is_some_and(|rec| matches!(rec.state(), InputNoteState::Committed { .. }));
    assert!(
        !is_committed,
        "the public note must not exist as committed without script pre-registration",
    );
    Ok(())
}

/// Compiles the counter contract library using the provided source manager so that all source
/// spans are registered in the same manager used by the client's executor.
pub(crate) fn counter_contract_library(source_manager: Arc<dyn SourceManagerSync>) -> Arc<Library> {
    let assembler = TransactionKernel::assembler_with_source_manager(source_manager.clone());
    let module = Module::parser(ModuleKind::Library)
        .parse_str(
            Path::new("external_contract::counter_contract"),
            COUNTER_CONTRACT,
            source_manager.clone(),
        )
        .map_err(|err| anyhow!(err))
        .unwrap();
    assembler
        .clone()
        .assemble_library([module])
        .map_err(|err| anyhow!(err))
        .unwrap()
}

fn get_network_note<T: Rng>(
    sender: AccountId,
    network_account: AccountId,
    source_manager: Arc<dyn SourceManagerSync>,
    rng: &mut T,
) -> Result<Note> {
    get_network_note_with_script(
        sender,
        network_account,
        INCR_NOTE_SCRIPT_CODE,
        source_manager,
        rng,
    )
}

pub(crate) fn get_network_note_with_script<T: Rng>(
    sender: AccountId,
    network_account: AccountId,
    script: &str,
    source_manager: Arc<dyn SourceManagerSync>,
    rng: &mut T,
) -> Result<Note> {
    let target = NetworkAccountTarget::new(network_account, NoteExecutionHint::Always)?;
    let attachment: NoteAttachment = target.into();
    let attachments = NoteAttachments::new(vec![attachment])?;
    let partial_metadata = PartialNoteMetadata::new(sender, NoteType::Public)
        .with_tag(NoteTag::with_account_target(network_account));

    let script = CodeBuilder::with_source_manager(source_manager.clone())
        .with_dynamically_linked_library(counter_contract_library(source_manager))?
        .compile_note_script(script)?;
    let recipient = NoteRecipient::new(
        Word::new([
            Felt::new(rng.random()),
            Felt::new(rng.random()),
            Felt::new(rng.random()),
            Felt::new(rng.random()),
        ]),
        script,
        NoteStorage::new(vec![])?,
    );

    let network_note =
        Note::with_attachments(NoteAssets::new(vec![])?, partial_metadata, recipient, attachments);
    Ok(network_note)
}

/// Watched-account flow against a network account:
///   - `client_1` deploys the counter as a network account and emits bump notes.
///   - `client_2` watches the network account via `import_watched_account_by_id` (no note tag).
///   - The node-driven counter increments are observed by `client_2` after `sync_state`.
pub async fn test_watch_network_account(client_config: ClientConfig) -> Result<()> {
    const BUMP_NOTE_NUMBER: u64 = 3;

    let (mut client_1, keystore_1) = client_config.clone().into_client().await?;
    let (mut client_2, _keystore_2) = ClientConfig::default()
        .with_rpc_endpoint(client_config.rpc_endpoint())
        .into_client()
        .await?;
    client_1.sync_state().await?;

    let network_account =
        deploy_counter_contract(&mut client_1, AccountStorageMode::Network).await?;
    let network_account_id = network_account.id();

    // Sanity: counter is 1 after deployment (deploy_counter_contract bumps it once).
    let counter_value = client_1
        .account_reader(network_account_id)
        .get_storage_item(COUNTER_SLOT_NAME.clone())
        .await
        .context("failed to find network account after deployment")?;
    assert_eq!(counter_value, Word::from([Felt::new(1), ZERO, ZERO, ZERO]));

    // client_2 starts watching the network account.
    client_2.import_watched_account_by_id(network_account_id).await?;

    let watched_record = client_2
        .test_store()
        .get_account(network_account_id)
        .await?
        .context("watched network account should be tracked in client_2's store")?;
    assert!(watched_record.is_watched(), "watched network account must be marked as watched");

    let tags = client_2.test_store().get_note_tags().await?;
    assert!(
        !tags
            .iter()
            .any(|t| matches!(t.source, NoteTagSource::Account(id) if id == network_account_id)),
        "watched network account must not register a per-account note tag",
    );

    let initial_watched_commitment =
        client_2.account_reader(network_account_id).commitment().await?;

    // client_1 emits BUMP_NOTE_NUMBER network notes targeted at the counter; the node will
    // consume them in subsequent blocks and bump the counter to 1 + BUMP_NOTE_NUMBER.
    let (native_account, ..) = insert_new_wallet(
        &mut client_1,
        AccountStorageMode::Public,
        &keystore_1,
        RPO_FALCON_SCHEME_ID,
    )
    .await?;

    let source_manager = client_1.source_manager();
    let mut network_notes = vec![];
    for _ in 0..BUMP_NOTE_NUMBER {
        let network_note = get_network_note(
            native_account.id(),
            network_account_id,
            source_manager.clone(),
            &mut client_1.rng(),
        )?;
        network_notes.push(network_note);
    }

    let tx_request = TransactionRequestBuilder::new().own_output_notes(network_notes).build()?;
    execute_tx_and_sync(&mut client_1, native_account.id(), tx_request).await?;

    // Poll the watched client until it observes the bumped counter.
    let expected_counter = Word::from([Felt::new(1 + BUMP_NOTE_NUMBER), ZERO, ZERO, ZERO]);
    let mut observed = false;
    for _ in 0..10 {
        wait_for_blocks(&mut client_1, 1).await;
        client_2.sync_state().await?;
        let counter = client_2
            .account_reader(network_account_id)
            .get_storage_item(COUNTER_SLOT_NAME.clone())
            .await?;
        if counter == expected_counter {
            observed = true;
            break;
        }
    }
    assert!(
        observed,
        "client_2 should observe the network account state advance via sync_state"
    );

    let source_commitment = client_1.account_reader(network_account_id).commitment().await?;
    let watched_commitment = client_2.account_reader(network_account_id).commitment().await?;
    assert_eq!(
        watched_commitment, source_commitment,
        "watched commitment should track source after node-driven bumps",
    );
    assert_ne!(
        watched_commitment, initial_watched_commitment,
        "watched commitment should have advanced",
    );

    Ok(())
}
