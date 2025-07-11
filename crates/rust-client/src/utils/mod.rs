mod accounts;
mod notes;
mod transactions;

use core::time::Duration;

use crate::{Client, ClientError, sync::SyncSummary};

pub const WAIT_TIME: Duration = Duration::from_secs(1);
pub const MAX_WAIT_LOOPS: u32 = 100;

impl Client {
    /// Waits until the condition is met, checking the sync state every `WAIT_TIME` seconds.
    /// If the condition is not met within `MAX_WAIT_LOOPS`, it returns an error.
    pub(crate) async fn wait_until(
        &mut self,
        mut condition: impl FnMut(&SyncSummary) -> bool,
    ) -> Result<(), ClientError> {
        for _ in 0..MAX_WAIT_LOOPS {
            let summary = self.sync_state().await?;

            if condition(&summary) {
                return Ok(());
            }

            std::thread::sleep(WAIT_TIME);
        }

        Err(ClientError::MaxWaitTimeExceeded)
    }

    /// Syncs until `amount_of_blocks` have been created onchain compared to client's sync height
    pub async fn wait_for_blocks(&mut self, amount_of_blocks: u32) -> Result<(), ClientError> {
        let current_block = self.get_sync_height().await?;
        let final_block = current_block + amount_of_blocks;

        self.wait_until(|summary| summary.block_num >= final_block).await
    }
}
