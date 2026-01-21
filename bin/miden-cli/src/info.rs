use std::fs;

use miden_client::Client;
use miden_client::account::AccountId;
use miden_client::auth::TransactionAuthenticator;
use miden_client::store::NoteFilter;

use super::config::CliConfig;
use crate::commands::account::DEFAULT_ACCOUNT_ID_KEY;
use crate::errors::CliError;

pub async fn print_client_info<AUTH: TransactionAuthenticator + Sync + 'static>(
    client: &Client<AUTH>,
) -> Result<(), CliError> {
    let config = CliConfig::from_system()?;

    println!("Client version: {}", env!("CARGO_PKG_VERSION"));
    print_config_stats(&config)?;
    print_client_stats(client).await
}

// HELPERS
// ================================================================================================
async fn print_client_stats<AUTH: TransactionAuthenticator + Sync + 'static>(
    client: &Client<AUTH>,
) -> Result<(), CliError> {
    println!("Block number: {}", client.get_sync_height().await?);
    println!("Tracked accounts: {}", client.get_account_headers().await?.len());
    println!("Expected notes: {}", client.get_input_notes(NoteFilter::Expected).await?.len());
    println!(
        "Default account: {}",
        client
            .get_setting(DEFAULT_ACCOUNT_ID_KEY.to_string())
            .await?
            .map_or("-".to_string(), AccountId::to_hex)
    );
    Ok(())
}

fn print_config_stats(config: &CliConfig) -> Result<(), CliError> {
    println!("Node address: {}", config.rpc.endpoint.0.host());
    let store_len = fs::metadata(config.store_filepath.clone())?.len();
    println!("Store size: {} kB", store_len / 1024);
    Ok(())
}
