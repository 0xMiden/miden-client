use anyhow::{Context, Result};
use miden_client::account::component::{AccountComponent, AuthRpoFalcon512};
use miden_client::account::{
    Account,
    AccountBuilder,
    AccountStorageMode,
    PartialAccount,
    PartialStorage,
    StorageMap,
    StorageSlot,
};
use miden_client::auth::AuthSecretKey;
use miden_client::crypto::rpo_falcon512::SecretKey;
use miden_client::rpc::domain::account::{AccountStorageRequirements, StorageMapKey};
use miden_client::testing::common::*;
use miden_client::transaction::{
    AdviceInputs,
    ForeignAccount,
    TransactionKernel,
    TransactionRequestBuilder,
};
use miden_client::{Felt, ScriptBuilder, Word};

use crate::tests::config::ClientConfig;

// FPI TESTS
// ================================================================================================

const MAP_KEY: [Felt; 4] = [Felt::new(15), Felt::new(15), Felt::new(15), Felt::new(15)];
const FPI_STORAGE_VALUE: [Felt; 4] =
    [Felt::new(9u64), Felt::new(12u64), Felt::new(18u64), Felt::new(30u64)];

pub async fn test_standard_fpi_public(client_config: ClientConfig) -> Result<()> {
    standard_fpi(AccountStorageMode::Public, client_config).await
}

pub async fn test_standard_fpi_private(client_config: ClientConfig) -> Result<()> {
    standard_fpi(AccountStorageMode::Private, client_config).await
}

pub async fn test_fpi_execute_program(client_config: ClientConfig) -> Result<()> {
    let (mut client, mut keystore) = client_config.clone().into_client().await?;
    client.sync_state().await?;

    // Deploy a foreign account
    let (foreign_account, proc_root) = deploy_foreign_account(
        &mut client,
        &mut keystore,
        AccountStorageMode::Public,
        format!(
            "export.get_fpi_map_item
                # map key
                push.{map_key}
                # item index
                push.0
                exec.::miden::account::get_map_item
                swapw dropw
            end",
            map_key = Word::from(MAP_KEY)
        ),
    )
    .await?;
    let foreign_account_id = foreign_account.id();
    println!("foreign account id {:?}", foreign_account_id.to_hex());

    let code = format!(
        "
        use.miden::tx
        use.miden::account
        begin
            # push the root of the `get_fpi_item` account procedure
            push.{proc_root}

            # push the foreign account id
            push.{account_id_suffix} push.{account_id_prefix}
            # => [foreign_id_prefix, foreign_id_suffix, FOREIGN_PROC_ROOT, storage_item_index]

            exec.tx::execute_foreign_procedure
        end
        ",
        account_id_prefix = foreign_account_id.prefix().as_u64(),
        account_id_suffix = foreign_account_id.suffix(),
    );

    let tx_script = client.script_builder().compile_tx_script(&code)?;
    client.sync_state().await?;

    // Wait for a couple of blocks so that the account gets committed
    wait_for_blocks(&mut client, 2).await;

    let storage_requirements =
        AccountStorageRequirements::new([(1u8, &[StorageMapKey::from(MAP_KEY)])]);

    // We create a new client here to force the creation of a new, fresh prover with no previous
    // MAST forest data.
    let (mut client2, keystore2) =
        ClientConfig::new(client_config.rpc_endpoint, client_config.rpc_timeout_ms)
            .into_client()
            .await?;

    let (wallet, ..) =
        insert_new_wallet(&mut client2, AccountStorageMode::Private, &keystore2).await?;

    let output_stack = client2
        .execute_program(
            wallet.id(),
            tx_script,
            AdviceInputs::default(),
            [ForeignAccount::public(foreign_account_id, storage_requirements)?].into(),
        )
        .await?;

    let mut expected_stack = [Felt::new(0); 16];
    expected_stack[3] = FPI_STORAGE_VALUE[0];
    expected_stack[2] = FPI_STORAGE_VALUE[1];
    expected_stack[1] = FPI_STORAGE_VALUE[2];
    expected_stack[0] = FPI_STORAGE_VALUE[3];

    assert_eq!(output_stack, expected_stack);
    Ok(())
}

pub async fn test_nested_fpi_calls(client_config: ClientConfig) -> Result<()> {
    let (mut client, mut keystore) = client_config.clone().into_client().await?;
    wait_for_node(&mut client).await;

    let (inner_foreign_account, inner_proc_root) = deploy_foreign_account(
        &mut client,
        &mut keystore,
        AccountStorageMode::Public,
        format!(
            "export.get_fpi_map_item
                # map key
                push.{map_key}
                # item index
                push.0
                exec.::miden::account::get_map_item
                swapw dropw
            end",
            map_key = Word::from(MAP_KEY)
        ),
    )
    .await?;
    let inner_foreign_account_id = inner_foreign_account.id();

    let (outer_foreign_account, outer_proc_root) = deploy_foreign_account(
        &mut client,
        &mut keystore,
        AccountStorageMode::Public,
        format!(
            "
            use.miden::tx
            export.get_fpi_map_item
                # push the hash of the `get_fpi_item` account procedure
                push.{inner_proc_root}

                # push the foreign account id
                push.{account_id_suffix} push.{account_id_prefix}
                # => [foreign_id_prefix, foreign_id_suffix, FOREIGN_PROC_ROOT, storage_item_index]

                exec.tx::execute_foreign_procedure

                # add one to the result of the foreign procedure call
                add.1
            end
            ",
            account_id_prefix = inner_foreign_account_id.prefix().as_u64(),
            account_id_suffix = inner_foreign_account_id.suffix(),
        ),
    )
    .await?;
    let outer_foreign_account_id = outer_foreign_account.id();

    println!("Calling FPI function inside a FPI function with new account");

    let tx_script = format!(
        "
        use.miden::tx
        use.miden::account
        begin
            # push the hash of the `get_fpi_item` account procedure
            push.{outer_proc_root}

            # push the foreign account id
            push.{account_id_suffix} push.{account_id_prefix}
            # => [foreign_id_prefix, foreign_id_suffix, FOREIGN_PROC_ROOT, storage_item_index]

            exec.tx::execute_foreign_procedure
            push.{fpi_value} add.1 assert_eqw
        end
        ",
        fpi_value = Word::from(FPI_STORAGE_VALUE),
        account_id_prefix = outer_foreign_account_id.prefix().as_u64(),
        account_id_suffix = outer_foreign_account_id.suffix(),
    );

    let tx_script = ScriptBuilder::new(true).compile_tx_script(tx_script)?;
    client.sync_state().await?;

    // Wait for a couple of blocks so that the account gets committed
    wait_for_blocks(&mut client, 2).await;

    // Create transaction request with FPI
    let builder = TransactionRequestBuilder::new().custom_script(tx_script);

    // We will require slot 0, key `MAP_KEY` as well as account proof
    let storage_requirements =
        AccountStorageRequirements::new([(1u8, &[StorageMapKey::from(MAP_KEY)])]);

    let foreign_accounts = [
        ForeignAccount::public(inner_foreign_account_id, storage_requirements.clone())?,
        ForeignAccount::public(outer_foreign_account_id, storage_requirements)?,
    ];

    let tx_request = builder.foreign_accounts(foreign_accounts).build()?;

    // We create a new client here to force the creation of a new, fresh prover with no previous
    // MAST forest data.
    let (mut client2, keystore2) =
        ClientConfig::new(client_config.rpc_endpoint, client_config.rpc_timeout_ms)
            .into_client()
            .await?;

    let (native_account, ..) =
        insert_new_wallet(&mut client2, AccountStorageMode::Public, &keystore2).await?;

    let tx_result = client2.new_transaction(native_account.id(), tx_request).await?;

    client2.submit_transaction(tx_result).await?;
    Ok(())
}

// HELPERS
// ================================================================================================

/// Tests the standard FPI functionality for the given storage mode.
///
/// This function sets up a foreign account with a custom component that retrieves a value from its
/// storage. It then deploys the foreign account and creates a native account to execute a
/// transaction that calls the foreign account's procedure via FPI. The test also verifies that the
/// foreign account's code is correctly cached after the transaction.
async fn standard_fpi(storage_mode: AccountStorageMode, client_config: ClientConfig) -> Result<()> {
    let (mut client, mut keystore) = client_config.clone().into_client().await?;
    wait_for_node(&mut client).await;

    let (foreign_account, proc_root) = deploy_foreign_account(
        &mut client,
        &mut keystore,
        storage_mode,
        format!(
            "export.get_fpi_map_item
                # map key
                push.{map_key}
                # item index
                push.0
                exec.::miden::account::get_map_item
                swapw dropw
            end",
            map_key = Word::from(MAP_KEY)
        ),
    )
    .await?;

    let foreign_account_id = foreign_account.id();

    println!("Calling FPI functions with new account");

    let tx_script = format!(
        "
        use.miden::tx
        use.miden::account
        begin
            # push the hash of the `get_fpi_item` account procedure
            push.{proc_root}

            # push the foreign account id
            push.{account_id_suffix} push.{account_id_prefix}
            # => [foreign_id_prefix, foreign_id_suffix, FOREIGN_PROC_ROOT, storage_item_index]

            exec.tx::execute_foreign_procedure
            push.{fpi_value} assert_eqw
        end
        ",
        fpi_value = Word::from(FPI_STORAGE_VALUE),
        account_id_prefix = foreign_account_id.prefix().as_u64(),
        account_id_suffix = foreign_account_id.suffix(),
    );

    let tx_script = ScriptBuilder::new(true).compile_tx_script(tx_script)?;
    client.sync_state().await?;

    // Wait for a couple of blocks so that the account gets committed
    wait_for_blocks(&mut client, 2).await;

    // Before the transaction there are no cached foreign accounts
    let foreign_accounts =
        client.test_store().get_foreign_account_code(vec![foreign_account_id]).await?;
    assert!(foreign_accounts.is_empty());

    // Create transaction request with FPI
    let builder = TransactionRequestBuilder::new().custom_script(tx_script);

    // We will require slot 0, key `MAP_KEY` as well as account proof
    let storage_requirements =
        AccountStorageRequirements::new([(1u8, &[StorageMapKey::from(MAP_KEY)])]);

    let foreign_account = if storage_mode == AccountStorageMode::Public {
        ForeignAccount::public(foreign_account_id, storage_requirements)
    } else {
        // Get current foreign account current state from the store (after 1st deployment tx)
        let foreign_account: Account = client
            .get_account(foreign_account_id)
            .await?
            .context("failed to find foreign account after deploiyng")?
            .into();

        // TODO: do into_parts
        let acc = PartialAccount::new(
            foreign_account.id(),
            foreign_account.nonce(),
            foreign_account.code().clone(),
            PartialStorage::new_full(foreign_account.storage().clone()),
            Default::default(),
            None,
        )?;

        ForeignAccount::private(acc)
    };

    let tx_request = builder.foreign_accounts([foreign_account?]).build()?;

    // We create a new client here to force the creation of a new, fresh prover with no previous
    // MAST forest data.
    let (mut client2, keystore2) =
        ClientConfig::new(client_config.rpc_endpoint, client_config.rpc_timeout_ms)
            .into_client()
            .await?;

    let (native_account, ..) =
        insert_new_wallet(&mut client2, AccountStorageMode::Public, &keystore2).await?;

    let tx_result = client2.new_transaction(native_account.id(), tx_request).await?;

    client2.submit_transaction(tx_result).await?;

    // After the transaction the foreign account should be cached (for public accounts only)
    if storage_mode == AccountStorageMode::Public {
        let foreign_accounts =
            client2.test_store().get_foreign_account_code(vec![foreign_account_id]).await?;
        assert_eq!(foreign_accounts.len(), 1);
    }
    Ok(())
}

/// Builds a foreign account with a custom component that exports the specified code.
///
/// # Returns
///
/// A tuple containing:
/// - `Account` - The constructed foreign account.
/// - `Word` - The seed used to initialize the account.
/// - `Word` - The procedure root of the custom component's procedure.
/// - `SecretKey` - The secret key used for authentication.
fn foreign_account_with_code(
    storage_mode: AccountStorageMode,
    code: String,
) -> Result<(Account, Word, SecretKey)> {
    // store our expected value on map from slot 0 (map key 15)
    let mut storage_map = StorageMap::new();
    storage_map.insert(MAP_KEY.into(), FPI_STORAGE_VALUE.into())?;

    let get_item_component = AccountComponent::compile(
        code,
        TransactionKernel::assembler(),
        vec![StorageSlot::Map(storage_map)],
    )
    .context("failed to compile foreign account component")?
    .with_supports_all_types();

    let secret_key = SecretKey::new();
    let auth_component = AuthRpoFalcon512::new(secret_key.public_key().to_commitment().into());

    let account = AccountBuilder::new(Default::default())
        .with_component(get_item_component.clone())
        .with_auth_component(auth_component)
        .storage_mode(storage_mode)
        .build()
        .context("failed to build foreign account")?;

    let proc_root = get_item_component
        .mast_forest()
        .procedure_digests()
        .next()
        .context("failed to get procedure root from component MAST forest")?;
    Ok((account, proc_root, secret_key))
}

/// Deploys a foreign account to the network with the specified code and storage mode. The account
/// is also inserted into the client and keystore.
///
/// # Returns
///
/// A tuple containing:
/// - `Account` - The deployed foreign account.
/// - `Word` - The procedure root of the foreign account.
async fn deploy_foreign_account(
    client: &mut TestClient,
    keystore: &mut TestClientKeyStore,
    storage_mode: AccountStorageMode,
    code: String,
) -> Result<(Account, Word)> {
    let (foreign_account, proc_root, secret_key) = foreign_account_with_code(storage_mode, code)?;
    let foreign_account_id = foreign_account.id();

    keystore
        .add_key(&AuthSecretKey::RpoFalcon512(secret_key))
        .with_context(|| "failed to add key to keystore")?;
    client.add_account(&foreign_account, false).await?;

    println!("Deploying foreign account");

    let tx = client
        .new_transaction(
            foreign_account_id,
            TransactionRequestBuilder::new()
                .build()
                .with_context(|| "failed to build transaction request")?,
        )
        .await?;
    let tx_id = tx.executed_transaction().id();
    client.submit_transaction(tx).await?;
    wait_for_tx(client, tx_id).await?;

    Ok((foreign_account, proc_root))
}
