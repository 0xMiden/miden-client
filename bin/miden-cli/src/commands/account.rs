use clap::Parser;
use comfy_table::{Cell, ContentArrangement, presets};
use miden_client::account::{Account, AccountId, AccountType, StorageSlotContent};
use miden_client::address::{Address, AddressInterface, RoutingParameters};
use miden_client::asset::Asset;
use miden_client::rpc::{GrpcClient, NodeRpcClient};
use miden_client::transaction::{AccountComponentInterface, AccountInterface};
use miden_client::{Client, PrettyPrint, ZERO};

use crate::config::CliConfig;
use crate::errors::CliError;
use crate::utils::{load_config_file, load_faucet_details_map, parse_account_id};
use crate::{client_binary_name, create_dynamic_table};

pub const DEFAULT_ACCOUNT_ID_KEY: &str = "default_account_id";

// ACCOUNT COMMAND
// ================================================================================================

/// View and manage accounts. Defaults to `list` command.
#[derive(Default, Debug, Clone, Parser)]
#[allow(clippy::option_option)]
pub struct AccountCmd {
    /// List all accounts monitored by this client (default action).
    #[arg(short, long, group = "action")]
    list: bool,
    /// Show details of the account for the specified ID or hex prefix.
    #[arg(short, long, group = "action", value_name = "ID")]
    show: Option<String>,
    /// When using --show, include the account code in the output.
    #[arg(long, requires = "show")]
    with_code: bool,
    /// Manages default account for transaction execution.
    ///
    /// If no ID is provided it will display the current default account ID.
    /// If "none" is provided it will remove the default account else it will set the default
    /// account to the provided ID.
    #[arg(short, long, group = "action", value_name = "ID")]
    default: Option<Option<String>>,
}

impl AccountCmd {
    pub async fn execute<AUTH>(&self, mut client: Client<AUTH>) -> Result<(), CliError> {
        let (cli_config, _) = load_config_file()?;
        match self {
            AccountCmd {
                list: false,
                show: Some(id),
                default: None,
                ..
            } => {
                let account_id = parse_account_id(&client, id).await?;
                show_account(client, account_id, &cli_config, self.with_code).await?;
            },
            AccountCmd {
                list: false,
                show: None,
                default: Some(id),
                ..
            } => {
                match id {
                    None => {
                        let default_account: AccountId = client
                            .get_setting(DEFAULT_ACCOUNT_ID_KEY.to_string())
                            .await?
                            .ok_or(CliError::Config(
                                "Default account".to_string().into(),
                                "No default account found in the client's store".to_string(),
                            ))?;
                        println!("Current default account ID: {default_account}");
                    },
                    Some(id) if id == "none" => {
                        client.remove_setting(DEFAULT_ACCOUNT_ID_KEY.to_string()).await?;
                        println!("Removing default account...");
                    },
                    Some(id) => {
                        let account_id: AccountId = parse_account_id(&client, id).await?;

                        // Check whether we're tracking that account
                        let (account, _) = client.try_get_account_header(account_id).await?;

                        client
                            .set_setting(DEFAULT_ACCOUNT_ID_KEY.to_string(), account.id())
                            .await?;
                        println!("Setting default account to {id}...");
                    },
                }
            },
            _ => {
                list_accounts(client).await?;
            },
        }
        Ok(())
    }
}

// LIST ACCOUNTS
// ================================================================================================

async fn list_accounts<AUTH>(client: Client<AUTH>) -> Result<(), CliError> {
    let accounts = client.get_account_headers().await?;

    let mut table =
        create_dynamic_table(&["Account ID", "Type", "Storage Mode", "Nonce", "Status"]);
    for (acc, _acc_seed) in &accounts {
        let status = client
            .get_account(acc.id())
            .await?
            .expect("Account should be in store")
            .status()
            .to_string();

        table.add_row(vec![
            acc.id().to_hex(),
            account_type_display_name(&acc.id())?,
            acc.id().storage_mode().to_string(),
            acc.nonce().as_int().to_string(),
            status,
        ]);
    }

    println!("{table}");
    Ok(())
}

// SHOW ACCOUNT
// ================================================================================================

pub async fn show_account<AUTH>(
    client: Client<AUTH>,
    account_id: AccountId,
    cli_config: &CliConfig,
    with_code: bool,
) -> Result<(), CliError> {
    let account = if let Some(account) = client.get_account(account_id).await? {
        // TODO: Show partial accounts through CLI
        account.try_into().map_err(|_| CliError::InvalidAccount(account_id))?
    } else {
        println!("Account {account_id} is not tracked by the client. Fetching from the network...",);

        let rpc_client =
            GrpcClient::new(&cli_config.rpc.endpoint.clone().into(), cli_config.rpc.timeout_ms);

        let fetched_account = rpc_client.get_account_details(account_id).await.map_err(|_| {
            CliError::Input(format!(
                "Unable to fetch account {account_id} from the network. It may not exist.",
            ))
        })?;

        let account: Option<Account> = fetched_account.into();

        account.ok_or(CliError::Input(format!(
            "Account {account_id} is private and not tracked by the client",
        )))?
    };

    print_summary_table(&account, &client, cli_config).await?;

    // Vault Table
    {
        let assets = account.vault().assets();
        let faucet_details_map = load_faucet_details_map()?;
        println!("Assets: ");

        let mut table = create_dynamic_table(&["Asset Type", "Faucet", "Amount"]);
        for asset in assets {
            let (asset_type, faucet, amount) = match asset {
                Asset::Fungible(fungible_asset) => {
                    let (faucet, amount) =
                        faucet_details_map.format_fungible_asset(&fungible_asset)?;
                    ("Fungible Asset", faucet, amount)
                },
                Asset::NonFungible(non_fungible_asset) => {
                    // TODO: Display non-fungible assets more clearly.
                    (
                        "Non Fungible Asset",
                        non_fungible_asset.faucet_id_prefix().to_hex(),
                        1.0.to_string(),
                    )
                },
            };
            table.add_row(vec![asset_type, &faucet, &amount.clone()]);
        }

        println!("{table}\n");
    }

    // Storage Table
    {
        let account_storage = account.storage();

        println!("Storage: \n");

        let mut table = create_dynamic_table(&["Slot Name", "Slot Type", "Value/Commitment"]);

        for entry in account_storage.slots() {
            let item = account_storage.get_item(entry.name()).map_err(|err| {
                CliError::Account(err, format!("failed to fetch slot {}", entry.name()))
            })?;

            // Last entry is reserved so I don't think the user cares about it. Also, to keep the
            // output smaller, if the [StorageSlot] is a value and it's 0 we assume it's not
            // initialized and skip it
            if matches!(entry.content(), StorageSlotContent::Value(_)) && item == [ZERO; 4].into() {
                continue;
            }

            let slot_type = match entry.content() {
                StorageSlotContent::Value(_) => "Value",
                StorageSlotContent::Map(_) => "Map",
            };
            table.add_row(vec![entry.name().as_str(), slot_type, &item.to_hex()]);
        }
        println!("{table}\n");
    }

    // Account code
    if with_code {
        println!("Code: \n");

        let mut table = create_dynamic_table(&["Code"]);
        table.add_row(vec![&account.code().to_pretty_string()]);
        println!("{table}");
    }

    Ok(())
}

// HELPERS
// ================================================================================================

/// Prints a summary table with account information.
async fn print_summary_table<AUTH>(
    account: &Account,
    client: &Client<AUTH>,
    cli_config: &CliConfig,
) -> Result<(), CliError> {
    let mut table = create_dynamic_table(&["Account Information"]);
    table
        .load_preset(presets::UTF8_HORIZONTAL_ONLY)
        .set_content_arrangement(ContentArrangement::DynamicFullWidth);

    table.add_row(vec![
        Cell::new("Address"),
        Cell::new(account_bech_32(account.id(), client, cli_config).await?),
    ]);
    table.add_row(vec![Cell::new("Account ID (hex)"), Cell::new(account.id().to_string())]);
    table.add_row(vec![
        Cell::new("Account Commitment"),
        Cell::new(account.commitment().to_string()),
    ]);
    table.add_row(vec![Cell::new("Type"), Cell::new(account_type_display_name(&account.id())?)]);
    table.add_row(vec![
        Cell::new("Storage mode"),
        Cell::new(account.id().storage_mode().to_string()),
    ]);
    table.add_row(vec![
        Cell::new("Code Commitment"),
        Cell::new(account.code().commitment().to_string()),
    ]);
    table.add_row(vec![Cell::new("Vault Root"), Cell::new(account.vault().root().to_string())]);
    table.add_row(vec![
        Cell::new("Storage Root"),
        Cell::new(account.storage().to_commitment().to_string()),
    ]);
    table.add_row(vec![Cell::new("Nonce"), Cell::new(account.nonce().as_int().to_string())]);

    println!("{table}\n");
    Ok(())
}

/// Returns a display name for the account type.
fn account_type_display_name(account_id: &AccountId) -> Result<String, CliError> {
    Ok(match account_id.account_type() {
        AccountType::FungibleFaucet => {
            let faucet_details_map = load_faucet_details_map()?;
            let token_symbol = faucet_details_map.get_token_symbol_or_default(account_id);

            format!("Fungible faucet (token symbol: {token_symbol})")
        },
        AccountType::NonFungibleFaucet => "Non-fungible faucet".to_string(),
        AccountType::RegularAccountImmutableCode => "Regular".to_string(),
        AccountType::RegularAccountUpdatableCode => "Regular (updatable)".to_string(),
    })
}

/// Sets the provided account ID as the default account in the client's store, if not set already.
pub(crate) async fn set_default_account_if_unset<AUTH>(
    client: &mut Client<AUTH>,
    account_id: AccountId,
) -> Result<(), CliError> {
    if client
        .get_setting::<AccountId>(DEFAULT_ACCOUNT_ID_KEY.to_string())
        .await?
        .is_some()
    {
        return Ok(());
    }

    client.set_setting(DEFAULT_ACCOUNT_ID_KEY.to_string(), account_id).await?;

    println!("Setting account {account_id} as the default account ID.");
    println!(
        "You can unset it with `{} account --default none`.",
        client_binary_name().display()
    );

    Ok(())
}

async fn account_bech_32<AUTH>(
    account_id: AccountId,
    client: &Client<AUTH>,
    cli_config: &CliConfig,
) -> Result<String, CliError> {
    let account_record = client.try_get_account(account_id).await?;
    let account: Account =
        account_record.try_into().map_err(|_| CliError::InvalidAccount(account_id))?;
    let account_interface: AccountInterface = (&account).into();

    let mut address = Address::new(account_id);
    if account_interface
        .components()
        .iter()
        .any(|c| matches!(c, AccountComponentInterface::BasicWallet))
    {
        address = address
            .with_routing_parameters(RoutingParameters::new(AddressInterface::BasicWallet))
            .map_err(|err| {
                CliError::Address(err, "Failed to set routing parameters".to_string())
            })?;
    }

    let encoded = address.encode(cli_config.rpc.endpoint.0.to_network_id());
    Ok(encoded)
}
