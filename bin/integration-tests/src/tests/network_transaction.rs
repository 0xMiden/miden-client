use std::sync::Arc;
use std::vec;

use anyhow::{Context, Result, anyhow};
use miden_client::account::component::AccountComponent;
use miden_client::account::{Account, AccountBuilder, AccountStorageMode, StorageSlot};
use miden_client::assembly::{
    Assembler,
    DefaultSourceManager,
    Library,
    LibraryPath,
    Module,
    ModuleKind,
};
use miden_client::note::NoteTag;
use miden_client::testing::common::{
    TestClient,
    create_test_client,
    execute_tx_and_sync,
    insert_new_wallet,
    wait_for_blocks,
    wait_for_tx,
};
use miden_client::testing::config::ClientConfig;
use miden_client::testing::note::NoteBuilder;
use miden_client::transaction::{OutputNote, TransactionKernel, TransactionRequestBuilder};
use miden_client::{Felt, ScriptBuilder, Word, ZERO};
use rand::RngCore;

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

/// Deploys a counter contract as a network account
async fn deploy_counter_contract(
    client: &mut TestClient,
    storage_mode: AccountStorageMode,
) -> Result<(Account, Library)> {
    let (acc, seed, library) = get_counter_contract_account(client, storage_mode).await?;

    client.add_account(&acc, Some(seed), false).await?;

    let mut script_builder = ScriptBuilder::new(true);
    script_builder.link_dynamic_library(&library)?;
    let tx_script = script_builder.compile_tx_script(
        "use.external_contract::counter_contract
        begin
            call.counter_contract::increment_count
        end",
    )?;

    // Build a transaction request with the custom script
    let tx_increment_request = TransactionRequestBuilder::new().custom_script(tx_script).build()?;

    // Execute the transaction locally
    let tx_result = client.new_transaction(acc.id(), tx_increment_request).await?;
    let tx_id = tx_result.executed_transaction().id();
    client.submit_transaction(tx_result).await?;
    wait_for_tx(client, tx_id).await?;

    Ok((acc, library))
}

async fn get_counter_contract_account(
    client: &mut TestClient,
    storage_mode: AccountStorageMode,
) -> Result<(Account, Word, Library)> {
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

    let (account, seed) = AccountBuilder::new(init_seed)
        .storage_mode(storage_mode)
        .with_component(counter_component)
        .with_auth_component(incr_nonce_auth)
        .build()
        .context("failed to build account with counter contract")?;

    let assembler: Assembler = TransactionKernel::assembler().with_debug_mode(true);
    let source_manager = Arc::new(DefaultSourceManager::default());
    let module = Module::parser(ModuleKind::Library)
        .parse_str(
            LibraryPath::new("external_contract::counter_contract")
                .context("failed to create library path for counter contract")?,
            COUNTER_CONTRACT,
            &source_manager,
        )
        .context("failed to parse counter contract module")?;
    let library = assembler
        .clone()
        .assemble_library([module])
        .context("failed to assemble counter contract library")?;

    Ok((account, seed, library))
}
// TESTS
// ================================================================================================

pub async fn counter_contract_ntx(client_config: ClientConfig) -> Result<()> {
    const BUMP_NOTE_NUMBER: u64 = 5;
    let (mut client, keystore) = create_test_client(client_config).await?;
    client.sync_state().await?;

    let (network_account, library) =
        deploy_counter_contract(&mut client, AccountStorageMode::Network).await?;

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

    let (native_account, _native_seed, _) =
        insert_new_wallet(&mut client, AccountStorageMode::Public, &keystore).await?;

    let assembler = TransactionKernel::assembler()
        .with_debug_mode(true)
        .with_dynamic_library(library)
        .map_err(|e| anyhow!(e))?;

    let mut network_notes = vec![];

    for _ in 0..BUMP_NOTE_NUMBER {
        network_notes.push(OutputNote::Full(
            NoteBuilder::new(native_account.id(), client.rng())
                .code(
                    "use.external_contract::counter_contract
                begin
                    call.counter_contract::increment_count
                end",
                )
                .tag(NoteTag::from_account_id(network_account.id()).into())
                .build(&assembler)?,
        ));
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

pub async fn recall_note_before_ntx_consumes_it(client_config: ClientConfig) -> Result<()> {
    let (mut client, keystore) = create_test_client(client_config).await?;
    client.sync_state().await?;

    let (network_account, library) =
        deploy_counter_contract(&mut client, AccountStorageMode::Network).await?;

    let native_account = deploy_counter_contract(&mut client, AccountStorageMode::Public).await?.0;

    let wallet = insert_new_wallet(&mut client, AccountStorageMode::Public, &keystore).await?.0;

    let assembler = TransactionKernel::assembler()
        .with_debug_mode(true)
        .with_dynamic_library(library)
        .map_err(|e| anyhow!(e))?;

    let network_note = NoteBuilder::new(wallet.id(), client.rng())
        .code(
            "use.external_contract::counter_contract
            begin
                call.counter_contract::increment_count
            end",
        )
        .tag(NoteTag::from_account_id(network_account.id()).into())
        .build(&assembler)?;

    // Prepare both transactions
    let tx_request = TransactionRequestBuilder::new()
        .own_output_notes(vec![OutputNote::Full(network_note.clone())])
        .build()?;

    let bump_transaction = client.new_transaction(wallet.id(), tx_request).await?;
    client.testing_apply_transaction(bump_transaction.clone()).await?;

    let tx_request = TransactionRequestBuilder::new()
        .unauthenticated_input_notes(vec![(network_note, None)])
        .build()?;

    let consume_transaction = client.new_transaction(native_account.id(), tx_request).await?;

    let bump_proof = client.testing_prove_transaction(&bump_transaction).await?;
    let consume_proof = client.testing_prove_transaction(&consume_transaction).await?;

    // Submit both transactions
    client.testing_submit_proven_transaction(bump_proof).await?;
    client.testing_submit_proven_transaction(consume_proof).await?;

    client.testing_apply_transaction(consume_transaction).await?;

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
