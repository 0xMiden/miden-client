use std::sync::{Arc, LazyLock};
use std::vec;

use anyhow::{Context, Result, anyhow};
use miden_client::account::component::{
    AccountComponent,
    AccountComponentMetadata,
    BurnPolicyConfig,
    FungibleTokenMetadata,
    MintPolicyConfig,
    NetworkFungibleFaucet,
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
use miden_client::asset::{FungibleAsset, TokenSymbol};
use miden_client::auth::RPO_FALCON_SCHEME_ID;
use miden_client::crypto::FeltRng;
use miden_client::note::{
    MintNote,
    MintNoteStorage,
    NetworkAccountTarget,
    Note,
    NoteAssets,
    NoteAttachment,
    NoteExecutionHint,
    NoteFile,
    NoteId,
    NoteMetadata,
    NoteRecipient,
    NoteStorage,
    NoteTag,
    NoteType,
    P2idNoteStorage,
};
use miden_client::store::InputNoteState;
use miden_client::sync::NoteTagSource;
use miden_client::testing::common::{
    TestClient,
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

/// After a network account consumes a note (potentially in the same batch it was created),
/// the receiver's `InputNoteReader` should find it as consumed by that account. Validates
/// the erased-notes detection flow end-to-end against a real node.
pub async fn test_note_reader_finds_note_consumed_by_ntx(
    client_config: ClientConfig,
) -> Result<()> {
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

    client.sync_state().await?;

    let mut reader = client.input_note_reader(network_account_id);
    let mut found = false;
    while let Some(note) = reader.next().await? {
        if note.id() == note_id {
            assert_eq!(
                note.consumer_account(),
                Some(network_account_id),
                "consumer should be the network account"
            );
            found = true;
            break;
        }
    }

    assert!(found, "NoteReader should find the note consumed by the network account");

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

    // Deploy the network-storage fungible faucet owned by Alice. Minting is
    // gated note-side (the `mint_and_send` procedure checks that the MINT
    // note sender == the Ownable2Step owner), so the faucet only needs a
    // no-auth component that unconditionally increments its nonce — the
    // same pattern used by `deploy_counter_contract` for network-storage
    // accounts above.
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
    let token_metadata = FungibleTokenMetadata::builder(name, symbol, 10, max_supply)
        .build()
        .map_err(|e| anyhow!("failed to build fungible token metadata: {e}"))?;
    let faucet = AccountBuilder::new(init_seed)
        .account_type(AccountType::FungibleFaucet)
        .storage_mode(AccountStorageMode::Network)
        .with_auth_component(incr_nonce_auth)
        .with_component(token_metadata)
        .with_component(NetworkFungibleFaucet)
        .with_components(TokenPolicyManager::new(
            PolicyAuthority::OwnerControlled,
            MintPolicyConfig::OwnerOnly,
            BurnPolicyConfig::AllowAll,
        ))
        .with_component(Ownable2Step::new(alice.id()))
        .build_with_schema_commitment()
        .map_err(|e| anyhow!("failed to build network faucet: {e}"))?;
    client.add_account(&faucet, false).await?;

    // Commit the faucet's initial state on-chain via a trivial incr-nonce
    // tx submitted from the faucet itself; without this the node's NTX
    // builder has no knowledge of the faucet and cannot run the MINT note
    // against it.
    let deploy_script = CodeBuilder::with_source_manager(client.source_manager())
        .compile_tx_script(NOOP_TX_SCRIPT)
        .context("failed to compile faucet deploy tx script")?;
    let deploy_tx = TransactionRequestBuilder::new().custom_script(deploy_script).build()?;
    let deploy_tx_id = client.submit_new_transaction(faucet.id(), deploy_tx).await?;
    wait_for_tx(&mut client, deploy_tx_id).await?;

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
    let mint_note = MintNote::create(
        faucet.id(),
        alice.id(), // must equal the faucet owner; checked by mint_and_send
        mint_storage,
        target_ntx.into(),
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
    let metadata = NoteMetadata::new(sender, NoteType::Public)
        .with_tag(NoteTag::with_account_target(network_account))
        .with_attachment(attachment);

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

    let network_note = Note::new(NoteAssets::new(vec![])?, metadata, recipient);
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
