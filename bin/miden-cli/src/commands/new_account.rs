use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use miden_client::Client;
use miden_client::account::component::{
    AccountComponent, AccountComponentMetadata, InitStorageData, MIDEN_PACKAGE_EXTENSION,
};
use miden_client::account::{Account, AccountBuilder, AccountStorageMode, AccountType};
use miden_client::auth::{AuthRpoFalcon512, AuthSecretKey, TransactionAuthenticator};
use miden_client::transaction::TransactionRequestBuilder;
use miden_client::utils::Deserializable;
use miden_client::vm::{Package, SectionId};
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
///
/// # Authentication Components
///
/// If a package with an authentication component is provided via `-p`, it will be used for
/// the account. Otherwise, a default `RpoFalcon512` authentication component will be added
/// automatically.
///
/// Each account can only have one authentication component. If multiple packages contain
/// authentication components, an error will be returned. By default, authentication-related
/// packages are located in the `auth` subdir in your packages directory.
///
/// # Examples
///
/// Create an account with default Falcon auth:
/// ```bash
/// miden-client new-account --account-type regular-account-immutable-code -p basic-wallet
/// ```
///
/// Create an account with a custom auth component (e.g., NoAuth):
/// ```bash
/// miden-client new-account --account-type regular-account-immutable-code -p auth/no-auth -p basic-wallet
/// ```
#[derive(Debug, Parser, Clone)]
pub struct NewAccountCmd {
    /// Storage mode of the account.
    #[arg(value_enum, short, long, default_value_t = CliAccountStorageMode::Private)]
    pub storage_mode: CliAccountStorageMode,
    /// Account type to create.
    #[arg(long, value_enum)]
    pub account_type: CliAccountType,
    /// List of files specifying package files used to create account components for the
    /// account.
    #[arg(short, long)]
    pub packages: Vec<PathBuf>,
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

/// Separates account components into auth and regular components.
///
/// Returns a tuple of (`auth_component`, `regular_components`).
/// Returns an error if multiple auth components are found.
fn separate_auth_components(
    components: Vec<AccountComponent>,
) -> Result<(Option<AccountComponent>, Vec<AccountComponent>), CliError> {
    let mut auth_component: Option<AccountComponent> = None;
    let mut regular_components = Vec::new();

    for component in components {
        let auth_proc_count =
            component.get_procedures().into_iter().filter(|(_, is_auth)| *is_auth).count();

        match auth_proc_count {
            0 => regular_components.push(component),
            1 => {
                if auth_component.is_some() {
                    return Err(CliError::InvalidArgument(
                        "Multiple auth components found in packages. Only one auth component is allowed per account.".to_string()
                    ));
                }
                auth_component = Some(component);
            },
            _ => {
                return Err(CliError::InvalidArgument(
                    "Component has multiple auth procedures. Only one auth procedure is allowed per component.".to_string()
                ));
            },
        }
    }

    Ok((auth_component, regular_components))
}

/// Helper function to create the seed, initialize the account builder, add the given components,
/// and build the account.
///
/// If no auth component is detected in the packages, a Falcon-based auth component will be added.
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

    let mut builder = AccountBuilder::new(init_seed)
        .account_type(account_type)
        .storage_mode(storage_mode);

    // Process packages and separate auth components from regular components
    let account_components = process_packages(packages, &init_storage_data)?;
    let (auth_component, regular_components) = separate_auth_components(account_components)?;

    // Add the auth component (either from packages or default Falcon)
    let key_pair = if let Some(auth_component) = auth_component {
        debug!("Adding auth component from package");
        builder = builder.with_auth_component(auth_component);
        None
    } else {
        debug!("Adding default Falcon auth component");
        let kp = AuthSecretKey::new_rpo_falcon512_with_rng(client.rng());
        builder =
            builder.with_auth_component(AuthRpoFalcon512::new(kp.public_key().to_commitment()));
        Some(kp)
    };

    // Add all regular (non-auth) components
    for component in regular_components {
        builder = builder.with_component(component);
    }

    let account = builder
        .build()
        .map_err(|err| CliError::Account(err, "failed to build account".into()))?;

    // Only add the key to the keystore if we generated a default key type (Falcon)
    if let Some(key_pair) = key_pair {
        keystore.add_key(&key_pair, &account.id()).map_err(CliError::KeyStore)?;
        println!("Generated and stored Falcon512 authentication key in keystore.");
    } else {
        println!("Using custom authentication component from package (no key generated).");
    }

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
                        mem_storew_be.4000 push.4000
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

    client.submit_new_transaction(account.id(), tx_request).await?;
    Ok(())
}

fn process_packages(
    packages: Vec<Package>,
    init_storage_data: &InitStorageData,
) -> Result<Vec<AccountComponent>, CliError> {
    let mut account_components = Vec::with_capacity(packages.len());

    for package in packages {
        let mut placeholders = init_storage_data.placeholders().clone();
        let mut map_entries = BTreeMap::new();

        let Some(component_metadata_section) = package.sections.iter().find(|section| {
            section.id.as_str() == (SectionId::ACCOUNT_COMPONENT_METADATA).as_str()
        }) else {
            continue;
        };

        let component_metadata = AccountComponentMetadata::read_from_bytes(
            &component_metadata_section.data,
        )
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
            if placeholders.contains_key(&placeholder_key) {
                // The use provided it through the TOML file, so we can skip it
                continue;
            }

            if let Some(entries) = init_storage_data.map_entries(&placeholder_key) {
                map_entries.insert(placeholder_key.clone(), entries.clone());
                continue;
            }

            let description = placeholder_type.description.unwrap_or("[No description]".into());
            println!(
                "Enter value for '{placeholder_key}' - {description} (type: {}): ",
                placeholder_type.r#type
            );
            std::io::stdout().flush()?;

            let mut input_value = String::new();
            std::io::stdin().read_line(&mut input_value)?;
            let input_value = input_value.trim();
            placeholders.insert(placeholder_key, input_value.to_string());
        }

        let account_component = AccountComponent::from_package_with_init_data(
            &package,
            &InitStorageData::new(placeholders, map_entries),
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
