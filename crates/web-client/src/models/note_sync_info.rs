use miden_client::rpc::domain::note::NoteSyncInfo as NativeNoteSyncInfo;
use js_export_macro::js_export;

use super::block_header::BlockHeader;
use super::committed_note::CommittedNote;
use super::merkle_path::MerklePath;

/// Represents the response data from `syncNotes`.
#[js_export]
pub struct NoteSyncInfo(NativeNoteSyncInfo);

#[js_export]
impl NoteSyncInfo {
    /// Returns the latest block number in the chain.
    #[js_export(js_name = "chainTip")]
    pub fn chain_tip(&self) -> u32 {
        self.0.chain_tip.as_u32()
    }

    /// Returns the block header associated with the matching notes.
    #[js_export(js_name = "blockHeader")]
    pub fn block_header(&self) -> BlockHeader {
        self.0.block_header.clone().into()
    }

    /// Returns the MMR path for the block header.
    #[js_export(js_name = "mmrPath")]
    pub fn mmr_path(&self) -> MerklePath {
        self.0.mmr_path.clone().into()
    }

    /// Returns the committed notes returned by the node.
    pub fn notes(&self) -> Vec<CommittedNote> {
        self.0.notes.iter().map(Into::into).collect()
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeNoteSyncInfo> for NoteSyncInfo {
    fn from(native_info: NativeNoteSyncInfo) -> Self {
        NoteSyncInfo(native_info)
    }
}
