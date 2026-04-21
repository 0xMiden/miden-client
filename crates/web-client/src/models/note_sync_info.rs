use js_export_macro::js_export;
use miden_client::rpc::domain::note::{
    NoteSyncBlock as NativeNoteSyncBlock,
    NoteSyncInfo as NativeNoteSyncInfo,
};

use super::block_header::BlockHeader;
use super::committed_note::CommittedNote;
use super::merkle_path::MerklePath;

/// Represents a single block's worth of note sync data.
#[js_export]
pub struct NoteSyncBlock(NativeNoteSyncBlock);

#[js_export]
impl NoteSyncBlock {
    /// Returns the block header for this block.
    #[js_export(js_name = "blockHeader")]
    pub fn block_header(&self) -> BlockHeader {
        self.0.block_header.clone().into()
    }

    /// Returns the MMR path for the block header.
    #[js_export(js_name = "mmrPath")]
    pub fn mmr_path(&self) -> MerklePath {
        self.0.mmr_path.clone().into()
    }

    /// Returns the committed notes in this block.
    pub fn notes(&self) -> Vec<CommittedNote> {
        self.0.notes.values().map(Into::into).collect()
    }
}

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

    /// Returns the last block checked by the node. Used as a cursor for pagination.
    #[js_export(js_name = "blockTo")]
    pub fn block_to(&self) -> u32 {
        self.0.block_to.as_u32()
    }

    /// Returns the first block header associated with matching notes, if any.
    #[js_export(js_name = "blockHeader")]
    pub fn block_header(&self) -> Option<BlockHeader> {
        self.0.blocks.first().map(|block| block.block_header.clone().into())
    }

    /// Returns the first block MMR path associated with matching notes, if any.
    #[js_export(js_name = "mmrPath")]
    pub fn mmr_path(&self) -> Option<MerklePath> {
        self.0.blocks.first().map(|block| block.mmr_path.clone().into())
    }

    /// Returns the committed notes across all matching blocks.
    pub fn notes(&self) -> Vec<CommittedNote> {
        self.0
            .blocks
            .iter()
            .flat_map(|block| block.notes.values().cloned())
            .map(Into::into)
            .collect()
    }

    /// Returns the blocks containing matching notes.
    pub fn blocks(&self) -> Vec<NoteSyncBlock> {
        self.0.blocks.iter().map(|b| NoteSyncBlock(b.clone())).collect()
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeNoteSyncInfo> for NoteSyncInfo {
    fn from(native_info: NativeNoteSyncInfo) -> Self {
        NoteSyncInfo(native_info)
    }
}
