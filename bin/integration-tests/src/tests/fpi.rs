use anyhow::{Context, Result};
use miden_client::account::component::AccountComponent;
use miden_client::account::{
    Account,
    AccountBuilder,
    AccountStorageMode,
    PartialAccount,
    PartialStorage,
    StorageMap,
    StorageSlot,
    StorageSlotName,
};
use miden_client::auth::{
    AuthEcdsaK256Keccak,
    AuthRpoFalcon512,
    AuthSchemeId,
    AuthSecretKey,
    RPO_FALCON_SCHEME_ID,
};
use miden_client::keystore::FilesystemKeyStore;
use miden_client::rpc::domain::account::{AccountStorageRequirements, StorageMapKey};
use miden_client::testing::common::*;
use miden_client::transaction::{AdviceInputs, ForeignAccount, TransactionRequestBuilder};
use miden_client::{CodeBuilder, Felt, Word};

use crate::tests::config::ClientConfig;

// FPI TESTS
// ================================================================================================

const MAP_KEY: [Felt; 4] = [Felt::new(15), Felt::new(15), Felt::new(15), Felt::new(15)];
const MAP_SLOT_NAME: &str = "miden::testing::fpi::map";
const FPI_STORAGE_VALUE: [Felt; 4] =
    [Felt::new(9u64), Felt::new(12u64), Felt::new(18u64), Felt::new(30u64)];

pub async fn test_standard_fpi_public(client_config: ClientConfig) -> Result<()> {
    standard_fpi(AccountStorageMode::Public, client_config, RPO_FALCON_SCHEME_ID).await
}

pub async fn test_standard_fpi_private(client_config: ClientConfig) -> Result<()> {
    standard_fpi(AccountStorageMode::Private, client_config, RPO_FALCON_SCHEME_ID).await
}

pub async fn test_fpi_execute_program(client_config: ClientConfig) -> Result<()> {
    let (mut client, keystore) = client_config.clone().into_client().await?;
    client.sync_state().await?;

    // Deploy a foreign account
    let (foreign_account, proc_root) = deploy_foreign_account(
        &mut client,
        &keystore,
        AccountStorageMode::Public,
        format!(
            r#"
            const MAP_STORAGE_SLOT = word("{MAP_SLOT_NAME}")
            pub proc get_fpi_map_item
                # map key
                push.{map_key}

                # item slot
                push.MAP_STORAGE_SLOT[0..2]
                
                exec.::miden::active_account::get_map_item
                swapw dropw
            end"#,
            map_key = Word::from(MAP_KEY)
        ),
        RPO_FALCON_SCHEME_ID,
    )
    .await?;
    let foreign_account_id = foreign_account.id();
    let code = format!(
        "
        use miden::tx
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

    let tx_script = client.code_builder().compile_tx_script(&code)?;
    client.sync_state().await?;

    // Wait for a couple of blocks so that the account gets committed
    wait_for_blocks(&mut client, 2).await;

    let map_slot_name = StorageSlotName::new(MAP_SLOT_NAME).expect("slot name should be valid");
    let storage_requirements =
        AccountStorageRequirements::new([(map_slot_name, &[StorageMapKey::from(MAP_KEY)])]);

    // We create a new client here to force the creation of a new, fresh prover with no previous
    // MAST forest data.
    let (mut client2, keystore2) =
        ClientConfig::new(client_config.rpc_endpoint, client_config.rpc_timeout_ms)
            .into_client()
            .await?;

    let (wallet, ..) = insert_new_wallet(
        &mut client2,
        AccountStorageMode::Private,
        &keystore2,
        RPO_FALCON_SCHEME_ID,
    )
    .await?;

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
    let (mut client, keystore) = client_config.clone().into_client().await?;
    wait_for_node(&mut client).await;

    let (inner_foreign_account, inner_proc_root) = deploy_foreign_account(
        &mut client,
        &keystore,
        AccountStorageMode::Public,
        format!(
            r#"
            const STORAGE_MAP_SLOT = word("{MAP_SLOT_NAME}")
            pub proc get_fpi_map_item
                # map key
                push.{map_key}

                # push item slot
                push.STORAGE_MAP_SLOT[0..2]

                # get item
                exec.::miden::active_account::get_map_item
                swapw dropw
            end"#,
            map_key = Word::from(MAP_KEY)
        ),
        RPO_FALCON_SCHEME_ID,
    )
    .await?;
    let inner_foreign_account_id = inner_foreign_account.id();

    let (outer_foreign_account, outer_proc_root) = deploy_foreign_account(
        &mut client,
        &keystore,
        AccountStorageMode::Public,
        format!(
            "
            use miden::tx
            pub proc get_fpi_map_item
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
        RPO_FALCON_SCHEME_ID,
    )
    .await?;
    let outer_foreign_account_id = outer_foreign_account.id();

    println!("Calling FPI function inside a FPI function with new account");

    let tx_script = format!(
        "
        use miden::tx
        use miden::account
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

    let tx_script = client.code_builder().compile_tx_script(&tx_script)?;
    client.sync_state().await?;

    // Wait for a couple of blocks so that the account gets committed
    wait_for_blocks(&mut client, 2).await;

    // Create transaction request with FPI
    let builder = TransactionRequestBuilder::new().custom_script(tx_script);

    // We will require slot 0, key `MAP_KEY` as well as account proof
    let map_slot_name = StorageSlotName::new(MAP_SLOT_NAME).expect("slot name should be valid");
    let storage_requirements =
        AccountStorageRequirements::new([(map_slot_name, &[StorageMapKey::from(MAP_KEY)])]);

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

    let (native_account, ..) = insert_new_wallet(
        &mut client2,
        AccountStorageMode::Public,
        &keystore2,
        RPO_FALCON_SCHEME_ID,
    )
    .await?;

    _ = client2.submit_new_transaction(native_account.id(), tx_request).await?;

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
async fn standard_fpi(
    storage_mode: AccountStorageMode,
    client_config: ClientConfig,
    auth_scheme: AuthSchemeId,
) -> Result<()> {
    let (mut client, keystore) = client_config.clone().into_client().await?;
    wait_for_node(&mut client).await;

    let (foreign_account, proc_root) = deploy_foreign_account(
        &mut client,
        &keystore,
        storage_mode,
        format!(
            r#"
            const STORAGE_MAP_SLOT = word("{MAP_SLOT_NAME}")
            pub proc get_fpi_map_item
                # map key
                push.{map_key}

                # push item slot name 
                push.STORAGE_MAP_SLOT[0..2]
                
                exec.::miden::active_account::get_map_item
                swapw dropw
            end"#,
            map_key = Word::from(MAP_KEY)
        ),
        auth_scheme,
    )
    .await?;

    let foreign_account_id = foreign_account.id();

    println!("Calling FPI functions with new account");

    let tx_script = format!(
        "
        use miden::tx
        use miden::account
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

    let tx_script = client.code_builder().compile_tx_script(&tx_script)?;
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
    let map_slot_name = StorageSlotName::new(MAP_SLOT_NAME).expect("slot name should be valid");
    let storage_requirements =
        AccountStorageRequirements::new([(map_slot_name, &[StorageMapKey::from(MAP_KEY)])]);

    let foreign_account = if storage_mode == AccountStorageMode::Public {
        ForeignAccount::public(foreign_account_id, storage_requirements)
    } else {
        // Get current foreign account current state from the store (after 1st deployment tx)
        let foreign_account: Account = client
            .get_account(foreign_account_id)
            .await?
            .context("failed to find foreign account after deploiyng")?
            .try_into()?;

        let (id, _vault, storage, code, nonce, seed) = foreign_account.into_parts();
        let acc = PartialAccount::new(
            id,
            nonce,
            code,
            PartialStorage::new_full(storage),
            Default::default(),
            seed,
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

    let (native_account, ..) = insert_new_wallet(
        &mut client2,
        AccountStorageMode::Public,
        &keystore2,
        RPO_FALCON_SCHEME_ID,
    )
    .await?;

    let block_before_wait = client2.get_sync_height().await.unwrap();
    wait_for_blocks_no_sync(&mut client2, 2).await;

    // Second client should be able to submit a transaction
    // Without being synced to latest state
    let _ = client2.submit_new_transaction(native_account.id(), tx_request).await?;

    // After the transaction the foreign account should be cached (for public accounts only)
    if storage_mode == AccountStorageMode::Public {
        let foreign_accounts =
            client2.test_store().get_foreign_account_code(vec![foreign_account_id]).await?;
        assert_eq!(foreign_accounts.len(), 1);
    }

    let block_after_wait = client2.get_sync_height().await.unwrap();

    // Submitted transaction should not have provoked a sync
    assert_eq!(block_before_wait, block_after_wait);

    client2.sync_state().await?;
    let block_after_sync = client2.get_sync_height().await.unwrap();

    // After syncing with the network, the client should be synced to the latest block
    assert!(block_after_wait < block_after_sync);

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
/// - `AuthSecretKey` - The secret key used for authentication.
fn foreign_account_with_code(
    storage_mode: AccountStorageMode,
    code: String,
    auth_scheme: AuthSchemeId,
) -> Result<(Account, Word, AuthSecretKey)> {
    // store our expected value on map from slot 0 (map key 15)
    let mut storage_map = StorageMap::new();
    storage_map.insert(MAP_KEY.into(), FPI_STORAGE_VALUE.into())?;

    let map_slot_name = StorageSlotName::new(MAP_SLOT_NAME).expect("slot name should be valid");
    let map_slot = StorageSlot::with_map(map_slot_name, storage_map);
    let component_code = CodeBuilder::default()
        .compile_component_code("miden::testing::fpi_component", code)
        .context("failed to compile foreign account component code")?;
    let get_item_component = AccountComponent::new(component_code, vec![map_slot])
        .map_err(|err| anyhow::anyhow!(err))
        .context("failed to create foreign account component")?
        .with_supports_all_types();

    let (key_pair, auth_component) = match auth_scheme {
        AuthSchemeId::RpoFalcon512 => {
            let key_pair = AuthSecretKey::new_falcon512_rpo();
            let auth_component: AccountComponent =
                AuthRpoFalcon512::new(key_pair.public_key().to_commitment()).into();
            (key_pair, auth_component)
        },
        AuthSchemeId::EcdsaK256Keccak => {
            let key_pair = AuthSecretKey::new_ecdsa_k256_keccak();
            let auth_component: AccountComponent =
                AuthEcdsaK256Keccak::new(key_pair.public_key().to_commitment()).into();
            (key_pair, auth_component)
        },
        scheme => {
            return Err(anyhow::anyhow!(format!("Unsupported auth scheme ID {}", scheme.as_u8())));
        },
    };

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
    Ok((account, proc_root, key_pair))
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
    keystore: &FilesystemKeyStore,
    storage_mode: AccountStorageMode,
    code: String,
    auth_scheme: AuthSchemeId,
) -> Result<(Account, Word)> {
    let (foreign_account, proc_root, secret_key) =
        foreign_account_with_code(storage_mode, code, auth_scheme)?;
    let foreign_account_id = foreign_account.id();

    keystore.add_key(&secret_key).with_context(|| "failed to add key to keystore")?;
    client.add_account(&foreign_account, false).await?;

    println!("Deploying foreign account");

    let tx_id = client
        .submit_new_transaction(
            foreign_account_id,
            TransactionRequestBuilder::new()
                .build()
                .with_context(|| "failed to build transaction request")?,
        )
        .await?;
    wait_for_tx(client, tx_id).await?;

    // NOTE: We get the new account state here since the first transaction updates the nonce from
    // to 1
    let foreign_account: Account =
        client.get_account(foreign_account_id).await?.unwrap().try_into().unwrap();

    Ok((foreign_account, proc_root))
}
