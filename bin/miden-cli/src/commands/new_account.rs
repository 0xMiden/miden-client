use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::vec;

use clap::{Parser, ValueEnum};
use miden_client::Client;
use miden_client::account::component::{
    AccountComponent,
    AccountComponentTemplate,
    COMPONENT_TEMPLATE_EXTENSION,
    InitStorageData,
    StorageValueName,
};
use miden_client::account::{Account, AccountBuilder, AccountStorageMode, AccountType};
use miden_client::auth::{AuthSecretKey, TransactionAuthenticator};
use miden_client::crypto::rpo_falcon512::SecretKey;
use miden_client::transaction::TransactionRequestBuilder;
use miden_client::utils::Deserializable;
use miden_lib::account::auth::AuthRpoFalcon512;
use rand::RngCore;
use tracing::debug;

use crate::commands::account::maybe_set_default_account;
use crate::errors::CliError;
use crate::{CliKeyStore, client_binary_name, load_config_file};

// CLI TYPES
// ================================================================================================

/// Mirror enum for [`AccountStorageMode`] that enables parsing for CLI commands.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum CliAccountStorageMode {
    Private,
    Public,
}

impl From<CliAccountStorageMode> for AccountStorageMode {
    fn from(cli_mode: CliAccountStorageMode) -> Self {
        match cli_mode {
            CliAccountStorageMode::Private => AccountStorageMode::Private,
            CliAccountStorageMode::Public => AccountStorageMode::Public,
        }
    }
}

/// Mirror enum for [`AccountType`] that enables parsing for CLI commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum CliAccountType {
    FungibleFaucet,
    NonFungibleFaucet,
    RegularAccountImmutableCode,
    RegularAccountUpdatableCode,
}

impl From<CliAccountType> for AccountType {
    fn from(cli_type: CliAccountType) -> Self {
        match cli_type {
            CliAccountType::FungibleFaucet => AccountType::FungibleFaucet,
            CliAccountType::NonFungibleFaucet => AccountType::NonFungibleFaucet,
            CliAccountType::RegularAccountImmutableCode => AccountType::RegularAccountImmutableCode,
            CliAccountType::RegularAccountUpdatableCode => AccountType::RegularAccountUpdatableCode,
        }
    }
}

// NEW WALLET
// ================================================================================================

/// Creates a new wallet account and store it locally.
///
/// A wallet account exposes functionality to sign transactions and
/// manage asset transfers. Additionally, more component templates can be added by specifying
/// a list of component template files.
#[derive(Debug, Parser, Clone)]
pub struct NewWalletCmd {
    /// Storage mode of the account.
    #[arg(value_enum, short, long, default_value_t = CliAccountStorageMode::Private)]
    pub storage_mode: CliAccountStorageMode,
    /// Defines if the account code is mutable (by default it isn't mutable).
    #[arg(short, long)]
    pub mutable: bool,
    /// Optional list of files specifying additional components to add to the account.
    #[arg(short, long)]
    pub extra_components: Vec<PathBuf>,
    /// Optional file path to a TOML file containing a list of key/values used for initializing
    /// storage. Each of these keys should map to the templated storage values within the passed
    /// list of component templates. The user will be prompted to provide values for any keys not
    /// present in the init storage data file.
    #[arg(short, long)]
    pub init_storage_data_path: Option<PathBuf>,
    /// If set, the newly created wallet will be deployed to the network by submitting an
    /// authentication transaction.
    #[arg(long, default_value_t = false)]
    pub deploy: bool,
}

impl NewWalletCmd {
    pub async fn execute<AUTH: TransactionAuthenticator + Sync + 'static>(
        &self,
        mut client: Client<AUTH>,
        keystore: CliKeyStore,
    ) -> Result<(), CliError> {
        let mut component_template_paths = vec![PathBuf::from("basic-wallet")];
        component_template_paths.extend(self.extra_components.iter().cloned());

        // Choose account type based on mutability.
        let account_type = if self.mutable {
            AccountType::RegularAccountUpdatableCode
        } else {
            AccountType::RegularAccountImmutableCode
        };

        let new_account = create_client_account(
            &mut client,
            &keystore,
            account_type,
            self.storage_mode.into(),
            &component_template_paths,
            self.init_storage_data_path.clone(),
            self.deploy,
        )
        .await?;

        let (mut current_config, _) = load_config_file()?;

        println!("Successfully created new wallet.");
        println!(
            "To view account details execute {} account -s {}",
            client_binary_name().display(),
            new_account.id().to_hex()
        );

        maybe_set_default_account(&mut current_config, new_account.id())?;

        Ok(())
    }
}

// NEW ACCOUNT
// ================================================================================================

/// Creates a new account and saves it locally.
///
/// An account may comprise one or more components, each with its own storage and distinct
/// functionality.
#[derive(Debug, Parser, Clone)]
pub struct NewAccountCmd {
    /// Storage mode of the account.
    #[arg(value_enum, short, long, default_value_t = CliAccountStorageMode::Private)]
    pub storage_mode: CliAccountStorageMode,
    /// Account type to create.
    #[arg(long, value_enum)]
    pub account_type: CliAccountType,
    /// Optional list of files specifying additional component template files to add to the
    /// account.
    #[arg(short, long)]
    pub component_templates: Vec<PathBuf>,
    /// Optional file path to a TOML file containing a list of key/values used for initializing
    /// storage. Each of these keys should map to the templated storage values within the passed
    /// list of component templates. The user will be prompted to provide values for any keys not
    /// present in the init storage data file.
    #[arg(short, long)]
    pub init_storage_data_path: Option<PathBuf>,
    /// If set, the newly created account will be deployed to the network by submitting an
    /// authentication transaction.
    #[arg(long, default_value_t = false)]
    pub deploy: bool,
}

impl NewAccountCmd {
    pub async fn execute<AUTH: TransactionAuthenticator + Sync + 'static>(
        &self,
        mut client: Client<AUTH>,
        keystore: CliKeyStore,
    ) -> Result<(), CliError> {
        let new_account = create_client_account(
            &mut client,
            &keystore,
            self.account_type.into(),
            self.storage_mode.into(),
            &self.component_templates,
            self.init_storage_data_path.clone(),
            self.deploy,
        )
        .await?;

        println!("Successfully created new account.");
        println!(
            "To view account details execute {} account -s {}",
            client_binary_name().display(),
            new_account.id().to_hex()
        );

        Ok(())
    }
}

// HELPERS
// ================================================================================================

/// Reads component templates from the given file paths.
// TODO: IO errors should have more context
fn load_component_templates(paths: &[PathBuf]) -> Result<Vec<AccountComponentTemplate>, CliError> {
    let (cli_config, _) = load_config_file()?;
    let components_base_dir = &cli_config.component_template_directory;
    let mut templates = Vec::new();
    for path in paths {
        // Set extension to COMPONENT_TEMPLATE_EXTENSION in case user did not
        let path = if path.extension().is_none() {
            path.with_extension(COMPONENT_TEMPLATE_EXTENSION)
        } else {
            path.clone()
        };
        let bytes = fs::read(components_base_dir.join(path))?;
        let template = AccountComponentTemplate::read_from_bytes(&bytes).map_err(|e| {
            CliError::AccountComponentError(
                Box::new(e),
                "failed to read account component template".into(),
            )
        })?;
        templates.push(template);
    }
    Ok(templates)
}

/// Loads the initialization storage data from an optional TOML file.
/// If None is passed, an empty object is returned.
fn load_init_storage_data(path: Option<PathBuf>) -> Result<InitStorageData, CliError> {
    if let Some(path) = path {
        let mut contents = String::new();
        File::open(path).and_then(|mut f| f.read_to_string(&mut contents))?;
        InitStorageData::from_toml(&contents).map_err(|err| CliError::Internal(Box::new(err)))
    } else {
        Ok(InitStorageData::default())
    }
}

/// Helper function to create the seed, initialize the account builder, add the given components,
/// and build the account.
///
/// The created account will have a Falcon-based auth component, additional to any specified
/// component.
async fn create_client_account<AUTH: TransactionAuthenticator + Sync + 'static>(
    client: &mut Client<AUTH>,
    keystore: &CliKeyStore,
    account_type: AccountType,
    storage_mode: AccountStorageMode,
    component_template_paths: &[PathBuf],
    init_storage_data_path: Option<PathBuf>,
    deploy: bool,
) -> Result<Account, CliError> {
    if component_template_paths.is_empty() {
        return Err(CliError::InvalidArgument(
            "account must contain at least one component".into(),
        ));
    }

    // Load the component templates and initialization storage data.
    debug!("Loading component templates...");
    let component_templates = load_component_templates(component_template_paths)?;
    debug!("Loaded {} component templates", component_templates.len());
    debug!("Loading initialization storage data...");
    let init_storage_data = load_init_storage_data(init_storage_data_path)?;
    debug!("Loaded initialization storage data");

    let mut init_seed = [0u8; 32];
    client.rng().fill_bytes(&mut init_seed);

    let key_pair = SecretKey::with_rng(client.rng());

    let mut builder = AccountBuilder::new(init_seed)
        .account_type(account_type)
        .storage_mode(storage_mode)
        .with_auth_component(AuthRpoFalcon512::new(key_pair.public_key()));

    // Process component templates and add them to the account builder.
    let account_components = process_component_templates(&component_templates, &init_storage_data)?;
    for component in account_components {
        builder = builder.with_component(component);
    }

    let (account, seed) = builder
        .build()
        .map_err(|err| CliError::Account(err, "failed to build account".into()))?;

    keystore
        .add_key(&AuthSecretKey::RpoFalcon512(key_pair))
        .map_err(CliError::KeyStore)?;

    client.add_account(&account, Some(seed), false).await?;

    if deploy {
        deploy_account(client, &account).await?;
    }

    Ok(account)
}

/// Submits a deploy transaction to the node for the specified account.
async fn deploy_account<AUTH: TransactionAuthenticator + Sync + 'static>(
    client: &mut Client<AUTH>,
    account: &Account,
) -> Result<(), CliError> {
    // Retrieve the auth procedure mast root pointer and call it in the transaction script.
    // We only use AuthRpoFalcon512 for the auth component so this may be overkill but it lets us
    // use different auth components in the future.
    let auth_procedure_mast_root = account.code().get_procedure_by_index(0).mast_root();

    let auth_script = client
        .script_builder()
        .compile_tx_script(
            "
                    begin
                        # [AUTH_PROCEDURE_MAST_ROOT]
                        mem_storew.4000 push.4000
                        # [auth_procedure_mast_root_ptr]
                        dyncall
                    end",
        )
        .expect("Auth script should compile");

    let tx_request = TransactionRequestBuilder::new()
        .script_arg(*auth_procedure_mast_root)
        .custom_script(auth_script)
        .build()
        .map_err(|err| {
            CliError::Transaction(err.into(), "Failed to build deploy transaction".to_string())
        })?;

    let tx = client.new_transaction(account.id(), tx_request).await?;
    client.submit_transaction(tx).await?;
    Ok(())
}

/// Helper function to process extra component templates.
/// It reads user input for each placeholder in a component template.
fn process_component_templates(
    extra_components: &[AccountComponentTemplate],
    file_init_storage_data: &InitStorageData,
) -> Result<Vec<AccountComponent>, CliError> {
    let mut account_components = vec![];
    for component_template in extra_components {
        let mut init_storage_data: BTreeMap<StorageValueName, String> =
            file_init_storage_data.placeholders().clone();
        for (placeholder_key, placeholder_type) in
            component_template.metadata().get_placeholder_requirements()
        {
            if init_storage_data.contains_key(&placeholder_key) {
                // The use provided it through the TOML file, so we can skip it
                continue;
            }

            let description = placeholder_type.description.unwrap_or("[No description]".into());
            print!(
                "Enter value for '{placeholder_key}' - {description} (type: {}): ",
                placeholder_type.r#type
            );
            std::io::stdout().flush()?;

            let mut input_value = String::new();
            std::io::stdin().read_line(&mut input_value)?;
            let input_value = input_value.trim();
            init_storage_data.insert(placeholder_key, input_value.to_string());
        }

        let component = AccountComponent::from_template(
            component_template,
            &InitStorageData::new(init_storage_data),
        )
        .map_err(|e| CliError::Account(e, "error instantiating component from template".into()))?;

        account_components.push(component);
    }

    Ok(account_components)
}
