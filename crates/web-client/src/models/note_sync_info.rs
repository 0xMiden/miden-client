use miden_client::rpc::domain::note::{
    NoteSyncBlock as NativeNoteSyncBlock, NoteSyncInfo as NativeNoteSyncInfo,
};
use wasm_bindgen::prelude::*;

use super::block_header::BlockHeader;
use super::committed_note::CommittedNote;
use super::merkle_path::MerklePath;

/// Represents a single block's worth of note sync data.
#[wasm_bindgen]
pub struct NoteSyncBlock(NativeNoteSyncBlock);

#[wasm_bindgen]
impl NoteSyncBlock {
    /// Returns the block header for this block.
    #[wasm_bindgen(js_name = "blockHeader")]
    pub fn block_header(&self) -> BlockHeader {
        self.0.block_header.clone().into()
    }

    /// Returns the MMR path for the block header.
    #[wasm_bindgen(js_name = "mmrPath")]
    pub fn mmr_path(&self) -> MerklePath {
        self.0.mmr_path.clone().into()
    }

    /// Returns the committed notes in this block.
    pub fn notes(&self) -> Vec<CommittedNote> {
        self.0.notes.iter().map(Into::into).collect()
    }
}

/// Represents the response data from `syncNotes`.
#[wasm_bindgen]
pub struct NoteSyncInfo(NativeNoteSyncInfo);

#[wasm_bindgen]
impl NoteSyncInfo {
    /// Returns the start of the scanned block range.
    #[wasm_bindgen(js_name = "blockFrom")]
    pub fn block_from(&self) -> u32 {
        self.0.block_from.as_u32()
    }

    /// Returns the end of the scanned block range (chain tip when `block_to` was not specified).
    #[wasm_bindgen(js_name = "blockTo")]
    pub fn block_to(&self) -> u32 {
        self.0.block_to.as_u32()
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
