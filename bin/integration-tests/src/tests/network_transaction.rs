use std::sync::{Arc, LazyLock};
use std::vec;

use anyhow::{Context, Result, anyhow};
use miden_client::account::component::AccountComponent;
use miden_client::account::{Account, AccountBuilder, AccountId, AccountStorageMode, StorageSlot};
use miden_client::assembly::{DefaultSourceManager, Library, LibraryPath, Module, ModuleKind};
use miden_client::note::{
    Note,
    NoteAssets,
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
use miden_client::{Felt, ScriptBuilder, Word, ZERO};
use rand::{Rng, RngCore};

use crate::tests::config::ClientConfig;

// HELPERS
// ================================================================================================

const COUNTER_CONTRACT: &str = "
        use.miden::account
        use.std::sys

        # => []
        export.get_count
            push.0
            exec.account::get_item
            exec.sys::truncate_stack
        end

        # => []
        export.increment_count
            push.0
            # => [index]
            exec.account::get_item
            # => [count]
            push.1 add
            # => [count+1]
            push.0
            # [index, count+1]
            exec.account::set_item
            # => []
            exec.sys::truncate_stack
            # => []
        end";

const INCR_NONCE_AUTH_CODE: &str = "
    use.miden::account
    export.auth__basic
        exec.account::incr_nonce
        drop
    end
";

const INCR_SCRIPT_CODE: &str = "
    use.external_contract::counter_contract
    begin
        call.counter_contract::increment_count
    end
";

/// Deploys a counter contract as a network account
async fn deploy_counter_contract(
    client: &mut TestClient,
    storage_mode: AccountStorageMode,
) -> Result<Account> {
    let acc = get_counter_contract_account(client, storage_mode).await?;

    client.add_account(&acc, false).await?;

    let mut script_builder = ScriptBuilder::new(true);
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
    let counter_component = AccountComponent::compile(
        COUNTER_CONTRACT,
        TransactionKernel::assembler(),
        vec![StorageSlot::empty_value()],
    )
    .context("failed to compile counter contract component")?
    .with_supports_all_types();

    let incr_nonce_auth =
        AccountComponent::compile(INCR_NONCE_AUTH_CODE, TransactionKernel::assembler(), vec![])
            .context("failed to compile increment nonce auth component")?
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

    assert_eq!(
        client
            .get_account(network_account.id())
            .await?
            .context("failed to find network account after deployment")?
            .account()
            .storage()
            .get_item(0)?,
        Word::from([ZERO, ZERO, ZERO, Felt::new(1)])
    );

    let (native_account, ..) =
        insert_new_wallet(&mut client, AccountStorageMode::Public, &keystore).await?;

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
        a.storage().get_item(0)?,
        Word::from([ZERO, ZERO, ZERO, Felt::new(1 + BUMP_NOTE_NUMBER)])
    );
    Ok(())
}

pub async fn test_recall_note_before_ntx_consumes_it(client_config: ClientConfig) -> Result<()> {
    let (mut client, keystore) = client_config.into_client().await?;
    client.sync_state().await?;

    let network_account = deploy_counter_contract(&mut client, AccountStorageMode::Network).await?;

    let native_account = deploy_counter_contract(&mut client, AccountStorageMode::Public).await?;

    let wallet = insert_new_wallet(&mut client, AccountStorageMode::Public, &keystore).await?.0;

    let network_note = get_network_note(wallet.id(), network_account.id(), &mut client.rng())?;
    // Prepare both transactions
    let tx_request = TransactionRequestBuilder::new()
        .own_output_notes(vec![OutputNote::Full(network_note.clone())])
        .build()?;

    let mut bump_pipeline = client.execute_transaction(wallet.id(), tx_request).await?;
    let tx_update = bump_pipeline
        .get_transaction_update_with_height(client.get_sync_height().await?)
        .unwrap();
    client.apply_transaction(tx_update).await?;

    let tx_request = TransactionRequestBuilder::new()
        .unauthenticated_input_notes(vec![(network_note, None)])
        .build()?;

    let mut consume_pipeline = client.execute_transaction(native_account.id(), tx_request).await?;

    bump_pipeline.prove_transaction(client.prover()).await?;
    consume_pipeline.prove_transaction(client.prover()).await?;

    // Submit both transactions
    bump_pipeline.submit_proven_transaction().await?;
    consume_pipeline.submit_proven_transaction().await?;

    let tx_update = consume_pipeline.get_transaction_update().unwrap();
    client.apply_transaction(tx_update).await?;

    wait_for_blocks(&mut client, 2).await;

    // The network account should have original value
    assert_eq!(
        client
            .get_account(network_account.id())
            .await?
            .context("failed to find network account after recall test")?
            .account()
            .storage()
            .get_item(0)?,
        Word::from([ZERO, ZERO, ZERO, Felt::new(1)])
    );

    // The native account should have the incremented value
    assert_eq!(
        client
            .get_account(native_account.id())
            .await?
            .context("failed to find native account after recall test")?
            .account()
            .storage()
            .get_item(0)?,
        Word::from([ZERO, ZERO, ZERO, Felt::new(2)])
    );
    Ok(())
}

// Initialize the Basic Fungible Faucet library only once.
static COUNTER_CONTRACT_LIBRARY: LazyLock<Library> = LazyLock::new(|| {
    let assembler = TransactionKernel::assembler().with_debug_mode(true);
    let source_manager = Arc::new(DefaultSourceManager::default());
    let module = Module::parser(ModuleKind::Library)
        .parse_str(
            LibraryPath::new("external_contract::counter_contract")
                .context("failed to create library path for counter contract")
                .unwrap(),
            COUNTER_CONTRACT,
            &source_manager,
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
    let metadata = NoteMetadata::new(
        sender,
        NoteType::Public,
        NoteTag::from_account_id(network_account),
        NoteExecutionHint::Always,
        ZERO,
    )?;

    let script = ScriptBuilder::new(true)
        .with_dynamically_linked_library(&counter_contract_library())?
        .compile_note_script(INCR_SCRIPT_CODE)?;
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
