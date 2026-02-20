use miden_client::rpc::domain::note::NoteSyncInfo as NativeNoteSyncInfo;

use super::block_header::BlockHeader;
use super::committed_note::CommittedNote;
use super::merkle_path::MerklePath;
use crate::prelude::*;

/// Represents the response data from `syncNotes`.
#[bindings]
pub struct NoteSyncInfo(NativeNoteSyncInfo);

#[bindings]
impl NoteSyncInfo {
    #[bindings]
    pub fn chain_tip(&self) -> u32 {
        self.0.chain_tip.as_u32()
    }

    #[bindings]
    pub fn block_header(&self) -> BlockHeader {
        self.0.block_header.clone().into()
    }

    #[bindings]
    pub fn mmr_path(&self) -> MerklePath {
        self.0.mmr_path.clone().into()
    }

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
