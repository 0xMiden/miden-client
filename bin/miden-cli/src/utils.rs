use std::path::PathBuf;

use miden_client::Client;
use miden_client::account::AccountId;
use miden_client::address::{Address, AddressId};

use super::{CLIENT_CONFIG_FILE_NAME, get_account_with_id_prefix};
use crate::commands::account::DEFAULT_ACCOUNT_ID_KEY;
use crate::config::{CliConfig, get_global_miden_dir, get_local_miden_dir};
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
        let address = Address::decode(account_id)
            .map_err(|err| CliError::Input(format!("error parsing bech32 address: {err}")))?
            .1;
        match address.id() {
            AddressId::AccountId(account_id_address) => Ok(account_id_address),
            _ => Err(CliError::Input(format!(
                "Input account ID {address:?} is not an ID based address"
            ))),
        }
    }
}

/// Loads config file from .miden directory with priority: local miden directory first, then global
/// fallback.
///
/// This function will look for the configuration file at the .miden/miden-client.toml path in the
/// following order:
///   - Local miden directory in current working directory
///   - Global miden directory in home directory
///
/// Note: Relative paths in the config are resolved relative to the .miden directory.
///
/// # Deprecated
/// This function is deprecated in favor of `CliConfig::from_system()` which provides
/// the same functionality with a cleaner API.
pub(super) fn load_config_file() -> Result<(CliConfig, PathBuf), CliError> {
    let config = CliConfig::from_system()?;

    // Determine which config file was loaded by checking existence
    let local_miden_dir = get_local_miden_dir()?;
    let local_config_path = local_miden_dir.join(CLIENT_CONFIG_FILE_NAME);

    let config_path = if local_config_path.exists() {
        local_config_path
    } else {
        let global_miden_dir = get_global_miden_dir().map_err(|e| {
            CliError::Config(Box::new(e), "Failed to determine global config directory".to_string())
        })?;
        global_miden_dir.join(CLIENT_CONFIG_FILE_NAME)
    };

    Ok((config, config_path))
}

/// Checks if either local or global configuration file exists.
pub(super) fn config_file_exists() -> Result<bool, CliError> {
    let local_miden_dir = get_local_miden_dir()?;
    if local_miden_dir.join(CLIENT_CONFIG_FILE_NAME).exists() {
        return Ok(true);
    }

    let global_miden_dir = get_global_miden_dir().map_err(|e| {
        CliError::Config(Box::new(e), "Failed to determine global config directory".to_string())
    })?;

    Ok(global_miden_dir.join(CLIENT_CONFIG_FILE_NAME).exists())
}

/// Returns the faucet details map using the config file.
pub fn load_faucet_details_map() -> Result<FaucetDetailsMap, CliError> {
    let config = CliConfig::from_system()?;
    FaucetDetailsMap::new(config.token_symbol_map_filepath)
}
