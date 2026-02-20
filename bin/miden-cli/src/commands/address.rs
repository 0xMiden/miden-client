use std::str::FromStr;

use miden_client::Client;
use miden_client::address::{Address, AddressInterface, NetworkId, RoutingParameters};
use miden_client::note::NoteTag;

use crate::config::CliConfig;
use crate::errors::CliError;
use crate::utils::parse_account_id;
use crate::{Parser, Subcommand, create_dynamic_table};

#[derive(Debug, Clone)]
pub enum CliAddressInterface {
    BasicWallet,
}

impl FromStr for CliAddressInterface {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "BasicWallet" => Ok(CliAddressInterface::BasicWallet),
            other => Err(format!(
                "Invalid interface: {other}. Valid values are: BasicWallet, Unspecified",
            )),
        }
    }
}

impl From<CliAddressInterface> for AddressInterface {
    fn from(value: CliAddressInterface) -> Self {
        match value {
            CliAddressInterface::BasicWallet => AddressInterface::BasicWallet,
        }
    }
}

#[derive(Debug, Subcommand, Clone)]
pub enum AddressSubCommand {
    /// List all addresses an account can be referenced by
    List { account_id: Option<String> },
    /// Add a new address
    Add {
        /// Account to add
        account_id: String,
        /// Interface number for add/remove operations
        interface: CliAddressInterface,
        /// Optional tag length
        tag_len: Option<u8>,
    },
    /// Remove the given address
    Remove {
        /// Account that owns the address to remove
        account_id: String,
        /// Address to remove
        address: String,
    },
}

#[derive(Debug, Parser, Clone)]
#[command(about = "Manage account addresses")]
pub struct AddressCmd {
    #[clap(subcommand)]
    command: Option<AddressSubCommand>,
}

impl AddressCmd {
    pub async fn execute<AUTH>(&self, client: Client<AUTH>) -> Result<(), CliError> {
        match &self.command {
            Some(AddressSubCommand::List { account_id: Some(account_id) }) => {
                let cli_config = CliConfig::from_system()?;
                let network_id = cli_config.rpc.endpoint.0.to_network_id();
                list_account_addresses(client, account_id, network_id).await?;
            },
            Some(AddressSubCommand::Add { interface, account_id, tag_len }) => {
                add_address(client, account_id.clone(), interface.clone(), *tag_len).await?;
            },
            Some(AddressSubCommand::Remove { account_id, address }) => {
                remove_address(client, account_id.clone(), address.clone()).await?;
            },
            _ => {
                // List all addresses as default
                let cli_config = CliConfig::from_system()?;
                let network_id = cli_config.rpc.endpoint.0.to_network_id();
                list_all_addresses(client, network_id).await?;
            },
        }
        Ok(())
    }
}

// HELPERS
// ================================================================================================

fn print_account_addresses(account_id: &String, addresses: &Vec<Address>, network_id: &NetworkId) {
    println!("Addresses for AccountId {account_id}:");
    let mut table = create_dynamic_table(&["Address", "Interface"]);
    for address in addresses {
        let address_bech32 = address.encode(network_id.clone());
        let interface = match address.interface() {
            Some(interface) => interface.to_string(),
            None => "Unspecified".to_string(),
        };
        table.add_row(vec![address_bech32, interface]);
    }

    println!("{table}");
}

async fn list_all_addresses<AUTH>(
    client: Client<AUTH>,
    network_id: NetworkId,
) -> Result<(), CliError> {
    println!("Listing addresses for all accounts:\n");
    let accounts = client.get_account_headers().await?;
    for (acc_header, _) in accounts {
        let addresses = client
            .account_reader(acc_header.id())
            .addresses()
            .await
            .expect("account is expected to exist if retrieved");
        print_account_addresses(&acc_header.id().to_string(), &addresses, &network_id);
        println!();
    }
    Ok(())
}

async fn list_account_addresses<AUTH>(
    client: Client<AUTH>,
    account_id: &String,
    network_id: NetworkId,
) -> Result<(), CliError> {
    let id = parse_account_id(&client, account_id).await?;
    let addresses = client.account_reader(id).addresses().await.map_err(|_| {
        CliError::Input(format!("The account with id `{account_id}` does not exist"))
    })?;

    print_account_addresses(&id.to_hex(), &addresses, &network_id);
    Ok(())
}

async fn add_address<AUTH>(
    mut client: Client<AUTH>,
    account_id: String,
    interface: CliAddressInterface,
    tag_len: Option<u8>,
) -> Result<(), CliError> {
    let account_id = parse_account_id(&client, &account_id).await?;
    let interface = interface.into();
    let routing_params = match tag_len {
        Some(tag_len) => RoutingParameters::new(interface)
            .with_note_tag_len(tag_len)
            .map_err(|e| CliError::Address(e, String::new()))?,
        None => RoutingParameters::new(interface),
    };
    let address = Address::new(account_id).with_routing_parameters(routing_params);

    let note_tag = NoteTag::with_account_target(account_id);
    client.add_address(address, account_id).await?;

    println!("Address added: Account Id {account_id} - Note tag: {note_tag}");
    Ok(())
}

async fn remove_address<AUTH>(
    mut client: Client<AUTH>,
    account_id: String,
    address: String,
) -> Result<(), CliError> {
    let account_id = parse_account_id(&client, &account_id).await?;
    let (_, address) = Address::decode(&address).map_err(|e| CliError::Address(e, address))?;
    let note_tag = address.to_note_tag();

    println!("removing address - Account Id {account_id} - Note tag: {note_tag}");

    client.remove_address(address, account_id).await?;
    Ok(())
}
