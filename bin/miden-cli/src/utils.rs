use std::path::{Path, PathBuf};

use figment::Figment;
use figment::providers::{Format, Toml};
use miden_client::Client;
use miden_client::account::AccountId;
use miden_client::address::Address;

use super::{CLIENT_CONFIG_FILE_NAME, get_account_with_id_prefix};
use crate::commands::account::DEFAULT_ACCOUNT_ID_KEY;
use crate::config::{CliConfig, MIDEN_DIR};
use crate::errors::CliError;
use crate::faucet_details_map::FaucetDetailsMap;

pub(crate) const SHARED_TOKEN_DOCUMENTATION: &str = "There are two accepted formats for the asset:
- `<AMOUNT>::<FAUCET_ID>` where `<AMOUNT>` is in the faucet base units.
- `<AMOUNT>::<TOKEN_SYMBOL>` where `<AMOUNT>` is a decimal number representing the quantity of
the token (specified to the precision allowed by the token's decimals), and `<TOKEN_SYMBOL>`
is a symbol tracked in the token symbol map file.

For example, `100::0xabcdef0123456789` or `1.23::TST`";

/// Returns a tracked Account ID matching a hex string or the default one defined in the Client
/// config.
pub(crate) async fn get_input_acc_id_by_prefix_or_default<AUTH>(
    client: &Client<AUTH>,
    account_id: Option<String>,
) -> Result<AccountId, CliError> {
    let account_id_str = if let Some(account_id_prefix) = account_id {
        account_id_prefix
    } else {
        client
            .get_setting(DEFAULT_ACCOUNT_ID_KEY.to_string())
            .await?
            .map(AccountId::to_hex)
            .ok_or(CliError::Input("No input account ID nor default account defined".to_string()))?
    };

    parse_account_id(client, &account_id_str).await
}

/// Parses a user provided account ID string and returns the corresponding `AccountId`.
///
/// `account_id` can fall into three categories:
///
/// - It's a hex prefix of an account ID of an account tracked by the client.
/// - It's a full hex account ID.
/// - It's a full bech32 account ID.
///
/// # Errors
///
/// - Will return a `IdPrefixFetchError` if the provided account ID string can't be parsed as an
///   `AccountId` and doesn't correspond to an account tracked by the client either.
pub(crate) async fn parse_account_id<AUTH>(
    client: &Client<AUTH>,
    account_id: &str,
) -> Result<AccountId, CliError> {
    if account_id.starts_with("0x") {
        if let Ok(account_id) = AccountId::from_hex(account_id) {
            return Ok(account_id);
        }

        Ok(get_account_with_id_prefix(client, account_id)
        .await
        .map_err(|_| CliError::Input(format!("Input account ID {account_id} is neither a valid Account ID nor a hex prefix of a known Account ID")))?
        .id())
    } else {
        let address = Address::from_bech32(account_id)
            .map_err(|err| CliError::Input(format!("error parsing bech32 address: {err}")))?
            .1;
        match address {
            Address::AccountId(account_id_address) => Ok(account_id_address.id()),
            _ => Err(CliError::Input(format!(
                "Input account ID {address:?} is not an ID based address"
            ))),
        }
    }
}

/// Loads config file from .miden directory and returns it alongside its path.
///
/// This function will look for the configuration file at the .miden/miden-client.toml path.
/// If the path is relative, searches in parent directories all the way to the root as well.
///
/// Note: Relative paths in the config are resolved relative to the .miden directory.
pub(super) fn load_config_file() -> Result<(CliConfig, PathBuf), CliError> {
    let mut config_path = std::env::current_dir()?;
    config_path.push(MIDEN_DIR);
    config_path.push(CLIENT_CONFIG_FILE_NAME);

    let mut cli_config = load_config(config_path.as_path())?;

    // Resolve relative paths in the config relative to the .miden directory
    let config_dir = config_path.parent().unwrap();

    if cli_config.store_filepath.is_relative() {
        cli_config.store_filepath = config_dir.join(&cli_config.store_filepath);
    }
    if cli_config.secret_keys_directory.is_relative() {
        cli_config.secret_keys_directory = config_dir.join(&cli_config.secret_keys_directory);
    }
    if cli_config.token_symbol_map_filepath.is_relative() {
        cli_config.token_symbol_map_filepath =
            config_dir.join(&cli_config.token_symbol_map_filepath);
    }
    if cli_config.package_directory.is_relative() {
        cli_config.package_directory = config_dir.join(&cli_config.package_directory);
    }

    Ok((cli_config, config_path))
}

/// Loads the client configuration.
fn load_config(config_file: &Path) -> Result<CliConfig, CliError> {
    Figment::from(Toml::file(config_file)).extract().map_err(|err| {
        CliError::Config("failed to load config file".to_string().into(), err.to_string())
    })
}

/// Returns the faucet details map using the config file.
pub fn load_faucet_details_map() -> Result<FaucetDetailsMap, CliError> {
    let (config, _) = load_config_file()?;
    FaucetDetailsMap::new(config.token_symbol_map_filepath)
}
