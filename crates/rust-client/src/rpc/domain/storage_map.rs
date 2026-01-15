use alloc::string::ToString;
use alloc::vec::Vec;

use miden_protocol::Word;
use miden_protocol::account::StorageSlotName;
use miden_protocol::block::BlockNumber;

use crate::rpc::domain::MissingFieldHelper;
use crate::rpc::{RpcConversionError, RpcError, generated as proto};

// STORAGE MAP INFO
// ================================================================================================

/// Represents a `proto::rpc::SyncStorageMapsResponse` with fields converted into domain
/// types. Contains information of updated map slots in a given range of blocks specified on
/// request. Also provides the current chain tip while processing the request.
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

impl TryFrom<proto::rpc::SyncStorageMapsResponse> for StorageMapInfo {
    type Error = RpcError;

    fn try_from(value: proto::rpc::SyncStorageMapsResponse) -> Result<Self, Self::Error> {
        let pagination_info = value.pagination_info.ok_or(
            proto::rpc::SyncStorageMapsResponse::missing_field(stringify!(pagination_info)),
        )?;
        let chain_tip = pagination_info.chain_tip;
        let block_number = pagination_info.block_num;

        let updates = value
            .updates
            .into_iter()
            .map(TryInto::try_into)
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

/// Represents a `proto::rpc::StorageMapUpdate`
pub struct StorageMapUpdate {
    /// Block number in which the slot was updated.
    pub block_num: BlockNumber,
    /// Name of the storage slot.
    pub slot_name: StorageSlotName,
    /// The storage map key
    pub key: Word,
    /// The storage map value.
    pub value: Word,
}

// STORAGE MAP UPDATE CONVERSION
// ================================================================================================

impl TryFrom<proto::rpc::StorageMapUpdate> for StorageMapUpdate {
    type Error = RpcConversionError;

    fn try_from(value: proto::rpc::StorageMapUpdate) -> Result<Self, Self::Error> {
        let block_num = value.block_num;

        let slot_name = StorageSlotName::new(value.slot_name)
            .map_err(|err| RpcConversionError::InvalidField(err.to_string()))?;

        let key: Word = value
            .key
            .ok_or(proto::rpc::StorageMapUpdate::missing_field(stringify!(key)))?
            .try_into()?;

        let value: Word = value
            .value
            .ok_or(proto::rpc::StorageMapUpdate::missing_field(stringify!(value)))?
            .try_into()?;

        Ok(Self {
            block_num: block_num.into(),
            slot_name,
            key,
            value,
        })
    }
}
