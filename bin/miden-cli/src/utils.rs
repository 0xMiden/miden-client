use miden_client::account::AccountId;
use miden_client::address::{Address, AddressId};
use miden_client::asset::{FungibleAsset, NonFungibleDeltaAction};
use miden_client::transaction::{ExecutedTransaction, InputNote};
use miden_client::{Client, Word};

use super::{CLIENT_CONFIG_FILE_NAME, create_dynamic_table, get_account_with_id_prefix};
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
    let config = CliConfig::load()?;
    FaucetDetailsMap::new(config.token_symbol_map_filepath)
}

/// Prints the effects of an executed transaction: input notes, output notes, storage value
/// changes, storage map changes, vault changes, and the nonce change.
pub fn print_executed_transaction(executed_tx: &ExecutedTransaction) -> Result<(), CliError> {
    println!("The transaction will have the following effects:\n");

    let delta = executed_tx.account_delta();

    // INPUT NOTES
    let input_note_ids = executed_tx.input_notes().iter().map(InputNote::id).collect::<Vec<_>>();
    if input_note_ids.is_empty() {
        println!("No notes will be consumed.");
    } else {
        println!("The following notes will be consumed:");
        for input_note_id in input_note_ids {
            println!("\t- {}", input_note_id.to_hex());
        }
    }
    println!();

    // OUTPUT NOTES
    let output_notes: Vec<_> = executed_tx.output_notes().iter().collect();
    if output_notes.is_empty() {
        println!("No notes will be created as a result of this transaction.");
    } else {
        println!("{} notes will be created as a result of this transaction:", output_notes.len());
        for note in &output_notes {
            println!("\t- {}", note.id().to_hex());
        }
    }
    println!();

    // STORAGE VALUES
    if delta.storage().values().next().is_some() {
        let mut table = create_dynamic_table(&["Storage Slot", "Effect"]);
        for (slot, new_value) in delta.storage().values() {
            table.add_row(vec![slot.to_string(), format!("Updated ({})", new_value.to_hex())]);
        }
        println!("Storage changes:");
        println!("{table}");
    } else {
        println!("Account Storage will not be changed.");
    }

    // STORAGE MAPS
    if delta.storage().maps().next().is_some() {
        let mut table = create_dynamic_table(&["Storage Slot", "Map Key", "New Value"]);
        for (slot, map_delta) in delta.storage().maps() {
            for (key, value) in map_delta.entries() {
                table.add_row(vec![slot.to_string(), Word::from(*key).to_hex(), value.to_hex()]);
            }
        }
        println!("Storage map changes:");
        println!("{table}");
    }

    // VAULT
    if delta.vault().is_empty() {
        println!("Account Vault will not be changed.");
    } else {
        let faucet_details_map = load_faucet_details_map()?;
        let mut table = create_dynamic_table(&["Asset Type", "Faucet ID", "Amount"]);

        for (vault_key, amount) in delta.vault().fungible().iter() {
            let asset = FungibleAsset::new(vault_key.faucet_id(), amount.unsigned_abs())
                .map_err(CliError::Asset)?;
            let (faucet_fmt, amount_fmt) = faucet_details_map.format_fungible_asset(&asset)?;

            if amount.is_positive() {
                table.add_row(vec!["Fungible Asset", &faucet_fmt, &format!("+{amount_fmt}")]);
            } else {
                table.add_row(vec!["Fungible Asset", &faucet_fmt, &format!("-{amount_fmt}")]);
            }
        }

        for (asset, action) in delta.vault().non_fungible().iter() {
            match action {
                NonFungibleDeltaAction::Add => {
                    table.add_row(vec![
                        "Non Fungible Asset",
                        &asset.faucet_id().prefix().to_hex(),
                        "1",
                    ]);
                },
                NonFungibleDeltaAction::Remove => {
                    table.add_row(vec![
                        "Non Fungible Asset",
                        &asset.faucet_id().prefix().to_hex(),
                        "-1",
                    ]);
                },
            }
        }

        println!("Vault changes:");
        println!("{table}");
    }

    // NONCE
    println!("Nonce incremented by: {}.", delta.nonce_delta());

    Ok(())
}
