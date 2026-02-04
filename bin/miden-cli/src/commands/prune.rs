use std::io;

use clap::Parser;
use miden_client::Client;

use crate::CliKeyStore;
use crate::errors::CliError;

// PRUNE COMMAND
// ================================================================================================

#[derive(Debug, Clone, Parser)]
#[command(
    about = "Prune old account states from the client's store, keeping only the latest state per account"
)]
pub struct PruneCmd {
    /// Do not prompt for confirmation
    #[clap(long)]
    force: bool,
}

impl PruneCmd {
    pub async fn execute(&self, client: Client<CliKeyStore>) -> Result<(), CliError> {
        if !self.force {
            println!(
                "This will permanently delete old account states, keeping only the latest state \
                 per account."
            );
            println!("States referenced by pending transactions will be preserved.");
            println!("Are you sure you want to continue? (y/N)");

            let mut proceed_str = String::new();
            io::stdin().read_line(&mut proceed_str)?;
            if proceed_str.trim().to_lowercase() != "y" {
                println!("Operation cancelled.");
                return Ok(());
            }
        }

        println!("Pruning old account states...");

        let pruned_count = client.prune_account_history().await?;

        if pruned_count == 0 {
            println!("No old account states to prune.");
        } else {
            println!("Successfully pruned {pruned_count} old account state(s).");
        }

        Ok(())
    }
}
