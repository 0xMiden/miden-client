use std::env;
use std::ffi::OsString;
use std::sync::Arc;

use clap::{Parser, Subcommand};
use comfy_table::{Attribute, Cell, ContentArrangement, Table, presets};
use errors::CliError;
use miden_client::account::AccountHeader;
use miden_client::auth::TransactionAuthenticator;
use miden_client::builder::ClientBuilder;
use miden_client::keystore::FilesystemKeyStore;
use miden_client::note_transport::grpc::GrpcNoteTransportClient;
use miden_client::store::{NoteFilter as ClientNoteFilter, OutputNoteRecord};
use miden_client::{Client, ClientError, DebugMode, IdPrefixFetchError};
use miden_client_sqlite_store::ClientBuilderSqliteExt;
use rand::rngs::StdRng;
mod commands;
use commands::account::AccountCmd;
use commands::clear_config::ClearConfigCmd;
use commands::exec::ExecCmd;
use commands::export::ExportCmd;
use commands::import::ImportCmd;
use commands::init::InitCmd;
use commands::new_account::{NewAccountCmd, NewWalletCmd};
use commands::new_transactions::{ConsumeNotesCmd, MintCmd, MintFaucetCmd, SendCmd, SwapCmd};
use commands::notes::NotesCmd;
use commands::sync::SyncCmd;
use commands::tags::TagsCmd;
use commands::transactions::TransactionCmd;

use self::utils::{config_file_exists, load_config_file};
use crate::commands::address::AddressCmd;

pub type CliKeyStore = FilesystemKeyStore<StdRng>;

mod config;
mod errors;
mod faucet_details_map;
mod info;
mod utils;

/// Re-export `MIDEN_DIR` for use in tests
pub use config::MIDEN_DIR;

/// Config file name.
const CLIENT_CONFIG_FILE_NAME: &str = "miden-client.toml";

/// Client binary name.
///
/// If, for whatever reason, we fail to obtain the client's executable name,
/// then we simply display the standard "miden-client".
pub fn client_binary_name() -> OsString {
    std::env::current_exe()
        .inspect_err(|e| {
            eprintln!(
                "WARNING: Couldn't obtain the path of the current executable because of {e}.\
             Defaulting to miden-client."
            );
        })
        .and_then(|executable_path| {
            executable_path.file_name().map(std::ffi::OsStr::to_os_string).ok_or(
                std::io::Error::other("Couldn't obtain the file name of the current executable"),
            )
        })
        .unwrap_or(OsString::from("miden-client"))
}

/// Number of blocks that must elapse after a transaction’s reference block before it is marked
/// stale and discarded.
const TX_GRACEFUL_BLOCK_DELTA: u32 = 20;

/// Root CLI struct.
#[derive(Parser, Debug)]
#[command(
    name = "miden-client",
    about = "The Miden client",
    version,
    propagate_version = true,
    rename_all = "kebab-case"
)]
#[command(multicall(true))]
pub struct MidenClientCli {
    #[command(subcommand)]
    behavior: Behavior,
}

impl From<MidenClientCli> for Cli {
    fn from(value: MidenClientCli) -> Self {
        match value.behavior {
            Behavior::MidenClient { cli } => cli,
            Behavior::External(args) => Cli::parse_from(args).set_external(),
        }
    }
}

#[derive(Debug, Subcommand)]
#[command(rename_all = "kebab-case")]
enum Behavior {
    /// The Miden Client CLI.
    MidenClient {
        #[command(flatten)]
        cli: Cli,
    },

    /// Used when the Miden Client CLI is called under a different name, like
    /// when it is called from [Midenup](https://github.com/0xMiden/midenup).
    /// Vec<OsString> holds the "raw" arguments passed to the command line,
    /// analogous to `argv`.
    #[command(external_subcommand)]
    External(Vec<OsString>),
}

#[derive(Parser, Debug)]
#[command(name = "miden-client")]
pub struct Cli {
    /// Activates the executor's debug mode, which enables debug output for scripts
    /// that were compiled and executed with this mode.
    #[arg(short, long, default_value_t = false)]
    debug: bool,

    #[command(subcommand)]
    action: Command,

    /// Indicates whether the client's CLI is being called directly, or
    /// externally under an alias (like in the case of
    /// [Midenup](https://github.com/0xMiden/midenup).
    #[arg(skip)]
    #[allow(unused)]
    external: bool,
}

/// CLI actions.
#[derive(Debug, Parser)]
pub enum Command {
    Account(AccountCmd),
    NewAccount(NewAccountCmd),
    NewWallet(NewWalletCmd),
    Import(ImportCmd),
    Export(ExportCmd),
    Init(InitCmd),
    ClearConfig(ClearConfigCmd),
    Notes(NotesCmd),
    Sync(SyncCmd),
    /// View a summary of the current client state.
    Info,
    Tags(TagsCmd),
    Address(AddressCmd),
    #[command(name = "tx")]
    Transaction(TransactionCmd),
    Mint(MintCmd),
    /// Mint tokens from a tracked faucet account using the legacy flow.
    #[command(name = "mint-faucet")]
    MintFaucet(MintFaucetCmd),
    Send(SendCmd),
    Swap(SwapCmd),
    ConsumeNotes(ConsumeNotesCmd),
    Exec(ExecCmd),
}

/// CLI entry point.
impl Cli {
    pub async fn execute(&self) -> Result<(), CliError> {
        // Handle commands that don't require client initialization
        match &self.action {
            Command::Init(init_cmd) => {
                init_cmd.execute()?;
                return Ok(());
            },
            Command::ClearConfig(clear_config_cmd) => {
                clear_config_cmd.execute()?;
                return Ok(());
            },
            _ => {},
        }

        // Check if Client is not yet initialized => silently initialize the client
        if !config_file_exists()? {
            let init_cmd = InitCmd::default();
            init_cmd.execute()?;
        }

        // Define whether we want to use the executor's debug mode based on the env var and
        // the flag override
        let in_debug_mode = match env::var("MIDEN_DEBUG") {
            Ok(value) if value.to_lowercase() == "true" => DebugMode::Enabled,
            _ => DebugMode::Disabled,
        };

        // Create the client
        let (cli_config, _config_path) = load_config_file()?;

        let keystore = CliKeyStore::new(cli_config.secret_keys_directory.clone())
            .map_err(CliError::KeyStore)?;

        let mut builder = ClientBuilder::new()
            .sqlite_store(cli_config.store_filepath.clone())
            .grpc_client(&cli_config.rpc.endpoint.clone().into(), Some(cli_config.rpc.timeout_ms))
            .authenticator(Arc::new(keystore.clone()))
            .in_debug_mode(in_debug_mode)
            .tx_graceful_blocks(Some(TX_GRACEFUL_BLOCK_DELTA));

        if let Some(delta) = cli_config.max_block_number_delta {
            builder = builder.max_block_number_delta(delta);
        }

        if let Some(tl_config) = cli_config.note_transport {
            let client =
                GrpcNoteTransportClient::connect(tl_config.endpoint.clone(), tl_config.timeout_ms)
                    .await
                    .map_err(|e| CliError::from(ClientError::from(e)))?;
            builder = builder.note_transport(Arc::new(client));
        }

        let client = builder.build().await?;

        // Execute CLI command
        match &self.action {
            Command::Account(account) => account.execute(client).await,
            Command::NewWallet(new_wallet) => Box::pin(new_wallet.execute(client, keystore)).await,
            Command::NewAccount(new_account) => {
                Box::pin(new_account.execute(client, keystore)).await
            },
            Command::Import(import) => import.execute(client, keystore).await,
            Command::Init(_) | Command::ClearConfig(_) => Ok(()), // Already handled earlier
            Command::Info => info::print_client_info(&client).await,
            Command::Notes(notes) => Box::pin(notes.execute(client)).await,
            Command::Sync(sync) => sync.execute(client).await,
            Command::Tags(tags) => tags.execute(client).await,
            Command::Address(addresses) => addresses.execute(client).await,
            Command::Transaction(transaction) => transaction.execute(client).await,
            Command::Exec(execute_program) => Box::pin(execute_program.execute(client)).await,
            Command::Export(cmd) => cmd.execute(client, keystore).await,
            Command::Mint(mint) => Box::pin(mint.execute(client)).await,
            Command::MintFaucet(mint) => Box::pin(mint.execute(client)).await,
            Command::Send(send) => Box::pin(send.execute(client)).await,
            Command::Swap(swap) => Box::pin(swap.execute(client)).await,
            Command::ConsumeNotes(consume_notes) => Box::pin(consume_notes.execute(client)).await,
        }
    }

    fn set_external(mut self) -> Self {
        self.external = true;
        self
    }
}

pub fn create_dynamic_table(headers: &[&str]) -> Table {
    let header_cells = headers
        .iter()
        .map(|header| Cell::new(header).add_attribute(Attribute::Bold))
        .collect::<Vec<_>>();

    let mut table = Table::new();
    table
        .load_preset(presets::UTF8_FULL)
        .set_content_arrangement(ContentArrangement::DynamicFullWidth)
        .set_header(header_cells);

    table
}

/// Returns the client output note whose ID starts with `note_id_prefix`.
///
/// # Errors
///
/// - Returns [`IdPrefixFetchError::NoMatch`] if we were unable to find any note where
///   `note_id_prefix` is a prefix of its ID.
/// - Returns [`IdPrefixFetchError::MultipleMatches`] if there were more than one note found where
///   `note_id_prefix` is a prefix of its ID.
pub(crate) async fn get_output_note_with_id_prefix<AUTH: TransactionAuthenticator + Sync>(
    client: &Client<AUTH>,
    note_id_prefix: &str,
) -> Result<OutputNoteRecord, IdPrefixFetchError> {
    let mut output_note_records = client
        .get_output_notes(ClientNoteFilter::All)
        .await
        .map_err(|err| {
            tracing::error!("Error when fetching all notes from the store: {err}");
            IdPrefixFetchError::NoMatch(format!("note ID prefix {note_id_prefix}").to_string())
        })?
        .into_iter()
        .filter(|note_record| note_record.id().to_hex().starts_with(note_id_prefix))
        .collect::<Vec<_>>();

    if output_note_records.is_empty() {
        return Err(IdPrefixFetchError::NoMatch(
            format!("note ID prefix {note_id_prefix}").to_string(),
        ));
    }
    if output_note_records.len() > 1 {
        let output_note_record_ids =
            output_note_records.iter().map(OutputNoteRecord::id).collect::<Vec<_>>();
        tracing::error!(
            "Multiple notes found for the prefix {}: {:?}",
            note_id_prefix,
            output_note_record_ids
        );
        return Err(IdPrefixFetchError::MultipleMatches(
            format!("note ID prefix {note_id_prefix}").to_string(),
        ));
    }

    Ok(output_note_records
        .pop()
        .expect("input_note_records should always have one element"))
}

/// Returns the client account whose ID starts with `account_id_prefix`.
///
/// # Errors
///
/// - Returns [`IdPrefixFetchError::NoMatch`] if we were unable to find any account where
///   `account_id_prefix` is a prefix of its ID.
/// - Returns [`IdPrefixFetchError::MultipleMatches`] if there were more than one account found
///   where `account_id_prefix` is a prefix of its ID.
async fn get_account_with_id_prefix<AUTH>(
    client: &Client<AUTH>,
    account_id_prefix: &str,
) -> Result<AccountHeader, IdPrefixFetchError> {
    let mut accounts = client
        .get_account_headers()
        .await
        .map_err(|err| {
            tracing::error!("Error when fetching all accounts from the store: {err}");
            IdPrefixFetchError::NoMatch(
                format!("account ID prefix {account_id_prefix}").to_string(),
            )
        })?
        .into_iter()
        .filter(|(account_header, _)| account_header.id().to_hex().starts_with(account_id_prefix))
        .map(|(acc, _)| acc)
        .collect::<Vec<_>>();

    if accounts.is_empty() {
        return Err(IdPrefixFetchError::NoMatch(
            format!("account ID prefix {account_id_prefix}").to_string(),
        ));
    }
    if accounts.len() > 1 {
        let account_ids = accounts.iter().map(AccountHeader::id).collect::<Vec<_>>();
        tracing::error!(
            "Multiple accounts found for the prefix {}: {:?}",
            account_id_prefix,
            account_ids
        );
        return Err(IdPrefixFetchError::MultipleMatches(
            format!("account ID prefix {account_id_prefix}").to_string(),
        ));
    }

    Ok(accounts.pop().expect("account_ids should always have one element"))
}
