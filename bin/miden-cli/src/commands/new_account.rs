use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use miden_client::Client;
use miden_client::account::component::{
    AccountComponent,
    AccountComponentMetadata,
    InitStorageData,
    MIDEN_PACKAGE_EXTENSION,
};
use miden_client::account::{Account, AccountBuilder, AccountStorageMode, AccountType};
use miden_client::auth::{AuthRpoFalcon512, AuthSecretKey, TransactionAuthenticator};
use miden_client::crypto::rpo_falcon512::SecretKey;
use miden_client::transaction::TransactionRequestBuilder;
use miden_client::utils::Deserializable;
use miden_client::vm::Package;
use rand::RngCore;
use tracing::debug;

use crate::commands::account::set_default_account_if_unset;
use crate::config::CliConfig;
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
    /// Optional list of paths specifying additional components in the form of
    /// packages to add to the account.
    #[arg(short, long)]
    pub extra_packages: Vec<PathBuf>,
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
        let package_paths: Vec<PathBuf> = [PathBuf::from("basic-wallet")]
            .into_iter()
            .chain(self.extra_packages.clone().into_iter())
            .collect();

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
            &package_paths,
            self.init_storage_data_path.clone(),
            self.deploy,
        )
        .await?;

        println!("Successfully created new wallet.");
        println!(
            "To view account details execute {} account -s {}",
            client_binary_name().display(),
            new_account.id().to_hex()
        );

        set_default_account_if_unset(&mut client, new_account.id()).await?;

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
    /// List of files specifying packages files used to create an account components for the
    /// account.
    #[arg(short, long)]
    pub packages: Vec<PathBuf>,
    #[deprecated(
        since = "0.12.0",
        note = "Component templates were superseded my [miden_client::vm::Package].\
                This field is only kept to inform users about said change."
    )]
    #[arg(short, long, hide(true))]
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
        // We allow #deprecated here in order to inform the user about the
        // migration from account component templates to packages.
        #[allow(deprecated)]
        if !self.component_templates.is_empty() {
            return Err(CliError::Input(format!(
                "Detected use of -c/--component-templates flag.
Account component templates have been replaced by the use of packages.
To use packages, pass the -p flag instead, like so:
{} new-account -p <package-name.masp>
",
                client_binary_name().display()
            )));
        }

        let new_account = create_client_account(
            &mut client,
            &keystore,
            self.account_type.into(),
            self.storage_mode.into(),
            &self.packages,
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

/// Reads [[`miden_core::vm::Package`]]s from the given file paths.
fn load_packages(
    cli_config: &CliConfig,
    package_paths: &[PathBuf],
) -> Result<Vec<Package>, CliError> {
    let mut packages = Vec::with_capacity(package_paths.len());

    let packages_dir = &cli_config.package_directory;
    for path in package_paths {
        // If a user passes in a file with the `.masp` file extension, then we
        // leave the path as is; since it probably is a full path (this is the
        // case with cargo-miden for instance).
        let path = match path.extension() {
            None => {
                let path = path.with_extension(MIDEN_PACKAGE_EXTENSION);
                Ok(packages_dir.join(path))
            },
            Some(extension) => {
                if extension == OsStr::new(MIDEN_PACKAGE_EXTENSION) {
                    Ok(path.clone())
                } else {
                    let error = std::io::Error::new(
                        std::io::ErrorKind::InvalidFilename,
                        format!(
                            "{} has an invalid file extension: '{}'. \
                            Expected: {MIDEN_PACKAGE_EXTENSION}",
                            path.display(),
                            extension.display()
                        ),
                    );
                    Err(CliError::AccountComponentError(
                        Box::new(error),
                        format!("refuesed to read {}", path.display()),
                    ))
                }
            },
        }?;

        let bytes = fs::read(&path).map_err(|e| {
            CliError::AccountComponentError(
                Box::new(e),
                format!("failed to read Package file from {}", path.display()),
            )
        })?;

        let package = Package::read_from_bytes(&bytes).map_err(|e| {
            CliError::AccountComponentError(
                Box::new(e),
                format!("failed to deserialize Package in {}", path.display()),
            )
        })?;

        packages.push(package);
    }

    Ok(packages)
}

/// Loads the initialization storage data from an optional TOML file.
/// If None is passed, an empty object is returned.
fn load_init_storage_data(path: Option<&PathBuf>) -> Result<InitStorageData, CliError> {
    if let Some(path) = &path {
        let mut contents = String::new();
        File::open(path)
            .and_then(|mut f| f.read_to_string(&mut contents))
            .map_err(|err| {
                CliError::InitDataError(
                    Box::new(err),
                    format!("Failed to open init data  file {}", path.display()),
                )
            })?;

        InitStorageData::from_toml(&contents).map_err(|err| {
            CliError::InitDataError(
                Box::new(err),
                format!("Failed to deserialize init data from file {}", path.display()),
            )
        })
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
    package_paths: &[PathBuf],
    init_storage_data_path: Option<PathBuf>,
    deploy: bool,
) -> Result<Account, CliError> {
    if package_paths.is_empty() {
        return Err(CliError::InvalidArgument(format!(
            "Account must contain at least one component. To provide one, pass a package with the -p flag, like so:
{} -p <package_name>
            ", client_binary_name().display())));
    }

    // Load the component templates and initialization storage data.

    let (cli_config, _) = load_config_file()?;
    debug!("Loading packages...");
    let packages = load_packages(&cli_config, package_paths)?;
    debug!("Loaded {} packages", packages.len());
    debug!("Loading initialization storage data...");
    let init_storage_data = load_init_storage_data(init_storage_data_path.as_ref())?;
    debug!("Loaded initialization storage data");

    let mut init_seed = [0u8; 32];
    client.rng().fill_bytes(&mut init_seed);

    let key_pair = SecretKey::with_rng(client.rng());

    let mut builder = AccountBuilder::new(init_seed)
        .account_type(account_type)
        .storage_mode(storage_mode)
        .with_auth_component(AuthRpoFalcon512::new(key_pair.public_key()));

    // Process packages and add them to the account builder.
    let account_components = process_packages(packages, &init_storage_data)?;
    for component in account_components {
        builder = builder.with_component(component);
    }

    let account = builder
        .build()
        .map_err(|err| CliError::Account(err, "failed to build account".into()))?;

    keystore
        .add_key(&AuthSecretKey::RpoFalcon512(key_pair))
        .map_err(CliError::KeyStore)?;

    client.add_account(&account, false).await?;

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

fn process_packages(
    packages: Vec<Package>,
    init_storage_data: &InitStorageData,
) -> Result<Vec<AccountComponent>, CliError> {
    let mut account_components = Vec::with_capacity(packages.len());

    for package in packages {
        let mut init_storage_data = init_storage_data.placeholders().clone();

        let Some(ref component_metadata) = package.account_component_metadata_bytes else {
            continue;
        };

        let component_metadata = AccountComponentMetadata::read_from_bytes(component_metadata)
            .map_err(|err| {
                CliError::AccountComponentError(
                    Box::new(err),
                    format!(
                        "Failed to deserialize Account Component Metadata from package {}",
                        package.name
                    ),
                )
            })?;

        for (placeholder_key, placeholder_type) in component_metadata.get_placeholder_requirements()
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

        let account_component = AccountComponent::from_package_with_init_data(
            &package,
            &InitStorageData::new(init_storage_data),
        )
        .map_err(|e| {
            CliError::Account(
                e,
                format!("error instantiating component from Package {}", package.name),
            )
        })?;

        account_components.push(account_component);
    }

    Ok(account_components)
}
