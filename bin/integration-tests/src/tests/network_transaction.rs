use std::sync::{Arc, LazyLock};
use std::vec;

use anyhow::{Context, Result, anyhow};
use miden_client::account::component::AccountComponent;
use miden_client::account::{
    Account,
    AccountBuilder,
    AccountId,
    AccountStorageMode,
    StorageSlot,
    StorageSlotName,
};
use miden_client::assembly::{
    CodeBuilder,
    DefaultSourceManager,
    Library,
    Module,
    ModuleKind,
    Path,
};
use miden_client::auth::RPO_FALCON_SCHEME_ID;
use miden_client::note::{
    NetworkAccountTarget,
    Note,
    NoteAssets,
    NoteAttachment,
    NoteExecutionHint,
    NoteInputs,
    NoteMetadata,
    NoteRecipient,
    NoteTag,
    NoteType,
};
use miden_client::testing::common::{
    TestClient,
    execute_tx_and_sync,
    insert_new_wallet,
    wait_for_blocks,
    wait_for_tx,
};
use miden_client::transaction::{OutputNote, TransactionKernel, TransactionRequestBuilder};
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
    pub proc auth__basic
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

/// Deploys a counter contract as a network account
pub(crate) async fn deploy_counter_contract(
    client: &mut TestClient,
    storage_mode: AccountStorageMode,
) -> Result<Account> {
    let acc = get_counter_contract_account(client, storage_mode).await?;

    client.add_account(&acc, false).await?;

    let mut script_builder = CodeBuilder::new();
    script_builder.link_dynamic_library(&counter_contract_library())?;
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
    let counter_component = AccountComponent::new(counter_code, vec![counter_slot])
        .map_err(|err| anyhow::anyhow!(err))
        .context("failed to create counter contract component")?
        .with_supports_all_types();

    let incr_nonce_auth_code = CodeBuilder::default()
        .compile_component_code("miden::testing::incr_nonce_auth", INCR_NONCE_AUTH_CODE)
        .context("failed to compile increment nonce auth component code")?;
    let incr_nonce_auth = AccountComponent::new(incr_nonce_auth_code, vec![])
        .map_err(|err| anyhow::anyhow!(err))
        .context("failed to create increment nonce auth component")?
        .with_supports_all_types();

    let mut init_seed = [0u8; 32];
    client.rng().fill_bytes(&mut init_seed);

    let account = AccountBuilder::new(init_seed)
        .storage_mode(storage_mode)
        .with_component(counter_component)
        .with_auth_component(incr_nonce_auth)
        .build()
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

    let account: Account = client
        .get_account(network_account.id())
        .await?
        .context("failed to find network account after deployment")?
        .try_into()?;
    assert_eq!(
        account.storage().get_item(&COUNTER_SLOT_NAME)?,
        Word::from([ZERO, ZERO, ZERO, Felt::new(1)])
    );

    let (native_account, ..) =
        insert_new_wallet(&mut client, AccountStorageMode::Public, &keystore, RPO_FALCON_SCHEME_ID)
            .await?;

    let mut network_notes = vec![];

    for _ in 0..BUMP_NOTE_NUMBER {
        let network_note =
            get_network_note(native_account.id(), network_account.id(), &mut client.rng())?;
        network_notes.push(OutputNote::Full(network_note));
    }

    let tx_request = TransactionRequestBuilder::new().own_output_notes(network_notes).build()?;

    execute_tx_and_sync(&mut client, native_account.id(), tx_request).await?;

    wait_for_blocks(&mut client, 2).await;

    let a = client
        .test_rpc_api()
        .get_account_details(network_account.id())
        .await?
        .account()
        .cloned()
        .with_context(|| "account details not available")?;

    assert_eq!(
        a.storage().get_item(&COUNTER_SLOT_NAME)?,
        Word::from([ZERO, ZERO, ZERO, Felt::new(1 + BUMP_NOTE_NUMBER)])
    );
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

    let network_note = get_network_note(wallet.id(), network_account.id(), &mut client.rng())?;
    // Prepare both transactions
    let tx_request = TransactionRequestBuilder::new()
        .own_output_notes(vec![OutputNote::Full(network_note.clone())])
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
    let account: Account = client
        .get_account(network_account.id())
        .await?
        .context("failed to find network account after recall test")?
        .try_into()?;
    assert_eq!(
        account.storage().get_item(&COUNTER_SLOT_NAME)?,
        Word::from([ZERO, ZERO, ZERO, Felt::new(1)])
    );

    // The native account should have the incremented value
    let account: Account = client
        .get_account(native_account.id())
        .await?
        .context("failed to find native account after recall test")?
        .try_into()
        .unwrap();
    assert_eq!(
        account.storage().get_item(&COUNTER_SLOT_NAME)?,
        Word::from([ZERO, ZERO, ZERO, Felt::new(2)])
    );
    Ok(())
}

// Initialize the Basic Fungible Faucet library only once.
static COUNTER_CONTRACT_LIBRARY: LazyLock<Library> = LazyLock::new(|| {
    let assembler = TransactionKernel::assembler();
    let source_manager = Arc::new(DefaultSourceManager::default());
    let module = Module::parser(ModuleKind::Library)
        .parse_str(
            Path::new("external_contract::counter_contract"),
            COUNTER_CONTRACT,
            source_manager,
        )
        .map_err(|err| anyhow!(err))
        .unwrap();
    assembler
        .clone()
        .assemble_library([module])
        .map_err(|err| anyhow!(err))
        .unwrap()
});

/// Returns the Basic Fungible Faucet Library.
fn counter_contract_library() -> Library {
    COUNTER_CONTRACT_LIBRARY.clone()
}

fn get_network_note<T: Rng>(
    sender: AccountId,
    network_account: AccountId,
    rng: &mut T,
) -> Result<Note> {
    get_network_note_with_script(sender, network_account, INCR_SCRIPT_CODE, rng)
}

pub(crate) fn get_network_note_with_script<T: Rng>(
    sender: AccountId,
    network_account: AccountId,
    script: &str,
    rng: &mut T,
) -> Result<Note> {
    let target = NetworkAccountTarget::new(network_account, NoteExecutionHint::Always)?;
    let attachment: NoteAttachment = target.into();
    let metadata =
        NoteMetadata::new(sender, NoteType::Public, NoteTag::with_account_target(network_account))
            .with_attachment(attachment);

    let script = CodeBuilder::new()
        .with_dynamically_linked_library(counter_contract_library())?
        .compile_note_script(script)?;
    let recipient = NoteRecipient::new(
        Word::new([
            Felt::new(rng.random()),
            Felt::new(rng.random()),
            Felt::new(rng.random()),
            Felt::new(rng.random()),
        ]),
        script,
        NoteInputs::new(vec![])?,
    );

    let network_note = Note::new(NoteAssets::new(vec![])?, metadata, recipient);
    Ok(network_note)
}
