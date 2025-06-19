mod accounts;
mod notes;
mod transactions;

use core::time::Duration;

pub use accounts::*;
pub use notes::*;
pub use transactions::*;

use crate::{Client, ClientError, sync::SyncSummary};

pub const WAIT_TIME: Duration = Duration::from_secs(1);

/// Syncs until `amount_of_blocks` have been created onchain compared to client's sync height
pub async fn wait_for_blocks(
    client: &mut Client,
    amount_of_blocks: u32,
) -> Result<(), ClientError> {
    let current_block = client.get_sync_height().await?;
    let final_block = current_block + amount_of_blocks;

    client.wait_until(|summary| summary.block_num >= final_block).await
}

impl Client {
    /// Waits until the condition is met, checking the sync state every `WAIT_TIME` seconds.
    pub async fn wait_until(
        &mut self,
        mut condition: impl FnMut(&SyncSummary) -> bool,
    ) -> Result<(), ClientError> {
        loop {
            let summary = self.sync_state().await?;

            if condition(&summary) {
                return Ok(());
            }

            std::thread::sleep(WAIT_TIME);
        }
    }
}
