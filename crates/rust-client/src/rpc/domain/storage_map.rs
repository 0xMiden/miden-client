use alloc::vec::Vec;

use miden_objects::Word;
use miden_objects::block::BlockNumber;

use crate::rpc::domain::MissingFieldHelper;
use crate::rpc::{RpcError, generated as proto};

// STORAGE MAP INFO
// ================================================================================================

/// Represents a `proto::rpc_store::SyncStorageMapsRequest`
pub struct StorageMapInfo {
    /// Current chain tip
    pub chain_tip: BlockNumber,
    /// The block number of the last check included in this response.
    pub block_number: BlockNumber,
    /// The list of storage map updates.
    pub updates: Vec<StorageMapUpdate>,
}

// STORAGE MAP INFO CONVERSION
// ================================================================================================

impl TryFrom<proto::rpc_store::SyncStorageMapsResponse> for StorageMapInfo {
    type Error = RpcError;

    fn try_from(value: proto::rpc_store::SyncStorageMapsResponse) -> Result<Self, Self::Error> {
        let pagination_info = value.pagination_info.ok_or(
            proto::rpc_store::SyncStorageMapsResponse::missing_field(stringify!(pagination_info)),
        )?;
        let chain_tip = pagination_info.chain_tip;
        let block_number = pagination_info.block_num;

        let updates = value
            .updates
            .iter()
            .map(|update| (*update).try_into())
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            chain_tip: chain_tip.into(),
            block_number: block_number.into(),
            updates,
        })
    }
}

// STORAGE MAP UPDATE
// ================================================================================================

/// Represents a `proto::rpc_store::StorageMapUpdate`
pub struct StorageMapUpdate {
    /// Block number in which the slot was updated.
    pub block_num: BlockNumber,
    /// Slot index ([0..255]).
    pub slot_index: u32,
    /// The storage map key
    pub key: Word,
    /// The storage map value.
    pub value: Word,
}

// STORAGE MAP UPDATE CONVERSION
// ================================================================================================

impl TryFrom<proto::rpc_store::StorageMapUpdate> for StorageMapUpdate {
    type Error = RpcError;

    fn try_from(value: proto::rpc_store::StorageMapUpdate) -> Result<Self, Self::Error> {
        let block_num = value.block_num;

        let slot_index = value.slot_index;

        let key: Word = value
            .key
            .ok_or(proto::rpc_store::SyncStorageMapsResponse::missing_field(stringify!(key)))?
            .try_into()?;

        let value: Word = value
            .value
            .ok_or(proto::rpc_store::SyncStorageMapsResponse::missing_field(stringify!(value)))?
            .try_into()?;

        Ok(Self {
            block_num: block_num.into(),
            slot_index,
            key,
            value,
        })
    }
}
