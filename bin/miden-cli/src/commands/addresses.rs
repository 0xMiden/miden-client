use miden_client::account::{AccountId, AccountIdAddress, AddressInterface};
use miden_client::address::Address;
use miden_client::note::NoteExecutionMode;
use miden_client::{Client, Serializable};

use crate::errors::CliError;
use crate::utils::parse_account_id;
use crate::{Parser, create_dynamic_table};

#[derive(Debug, Parser, Clone)]
#[command(about = "Manage account addresses")]
pub struct AddressesCmd {
    /// List all addresses an account can be referenced by.
    #[arg(short, long, group = "action")]
    list: bool,

    /// Add a new address.
    #[arg(short, long, group = "action")]
    add: bool,

    /// Remove an existing address.
    #[arg(short, long, group = "action")]
    remove: bool,

    /// Hex representation of [`AccountId`]
    #[arg(required = true)]
    account_id: String,

    /// Interface number for add/remove operations
    #[arg(required_if_eq_any([("add", "true"), ("remove", "true")]))]
    interface: Option<String>,
}

impl AddressesCmd {
    pub async fn execute<AUTH>(&self, client: Client<AUTH>) -> Result<(), CliError> {
        match (self.add, self.remove, self.list) {
            (true, ..) => {
                // We can safely unwrap interface because it's required when add is true
                add_address(client, self.account_id.clone(), self.interface.clone().unwrap())
                    .await?;
            },
            (_, true, _) => {
                // We can safely unwrap interface because it's required when remove is true
                remove_address(client, self.account_id.clone(), self.interface.clone().unwrap())
                    .await?;
            },
            _ => {
                list_addresses(client, self.account_id.clone()).await?;
            },
        }
        Ok(())
    }
}

// HELPERS
// ================================================================================================
async fn list_addresses<AUTH>(client: Client<AUTH>, account_id: String) -> Result<(), CliError> {
    let id = parse_account_id(&client, &account_id).await?;
    let addresses = match client.get_account(id).await? {
        Some(account) => account.addresses().clone(),
        _ => {
            return Err(CliError::Input(format!(
                "The account with id `{account_id}` does not exist",
            )));
        },
    };

    println!("Addresses for AccountId {account_id}:");
    let mut table = create_dynamic_table(&["Address", "Interface"]);
    for address in addresses {
        let address_hex = hex::encode(address.to_bytes());
        let interface = match address.interface() {
            AddressInterface::Unspecified => "Unspecified".to_string(),
            AddressInterface::BasicWallet => "Basic Wallet".to_string(),
            _ => "Unknown Address Interface".to_string(),
        };

        table.add_row(vec![address_hex, interface]);
    }

    println!("{table}");

    Ok(())
}

fn build_address_from_cli_args(
    account_id: AccountId,
    interface: &str,
) -> Result<Address, CliError> {
    let interface = match interface {
        "unspecified" => AddressInterface::Unspecified,
        "basic_wallet" => AddressInterface::BasicWallet,
        _ => return Err(CliError::Input("Invalid interface input value".to_string())),
    };
    Ok(Address::AccountId(AccountIdAddress::new(account_id, interface)))
}

async fn add_address<AUTH>(
    mut client: Client<AUTH>,
    account_id: String,
    interface: String,
) -> Result<(), CliError> {
    let account_id = parse_account_id(&client, &account_id).await?;
    let address = build_address_from_cli_args(account_id, &interface)?;
    let execution_mode = match address.to_note_tag().execution_mode() {
        NoteExecutionMode::Local => "Local",
        NoteExecutionMode::Network => "Network",
    };
    println!(
        "adding address - Account Id {} - Execution mode: {}",
        account_id, execution_mode
    );

    client.add_address(address, account_id).await?;
    Ok(())
}

async fn remove_address<AUTH>(
    mut client: Client<AUTH>,
    account_id: String,
    interface: String,
) -> Result<(), CliError> {
    let account_id = parse_account_id(&client, &account_id).await?;
    let address = build_address_from_cli_args(account_id, &interface)?;
    let execution_mode = match address.to_note_tag().execution_mode() {
        NoteExecutionMode::Local => "Local",
        NoteExecutionMode::Network => "Network",
    };

    println!(
        "removing address - Account Id {} - Execution mode: {}",
        account_id, execution_mode
    );

    client.remove_address(address, account_id).await?;
    Ok(())
}
