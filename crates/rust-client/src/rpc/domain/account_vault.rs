use alloc::vec::Vec;

use miden_objects::Word;
use miden_objects::asset::Asset;
use miden_objects::block::BlockNumber;

use crate::rpc::domain::MissingFieldHelper;
use crate::rpc::{RpcError, generated as proto};

// ACCOUNT VAULT INFO
// ================================================================================================

pub struct AccountVaultInfo {
    /// Current chain tip
    pub chain_tip: BlockNumber,
    /// The block number of the last check included in this response.
    pub block_number: BlockNumber,
    /// List of asset updates for the account.
    pub updates: Vec<AccountVaultUpdate>,
}

// ACCOUNT VAULT CONVERSION
// ================================================================================================

impl TryFrom<proto::rpc_store::SyncAccountVaultResponse> for AccountVaultInfo {
    type Error = RpcError;

    fn try_from(value: proto::rpc_store::SyncAccountVaultResponse) -> Result<Self, Self::Error> {
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

// ACCOUNT VAULT UPDATE
// ================================================================================================

pub struct AccountVaultUpdate {
    /// Block number in which the slot was updated.
    pub block_num: BlockNumber,
    /// Asset value related to the vault key. If not present, the asset was removed from the vault.
    pub asset: Option<Asset>,
    /// Vault key associated with the asset.
    pub vault_key: Word,
}

// ACCOUNT VAULT UPDATE CONVERSION
// ================================================================================================

impl TryFrom<proto::primitives::Asset> for Asset {
    type Error = RpcError;

    fn try_from(value: proto::primitives::Asset) -> Result<Self, Self::Error> {
        let word: Word = value
            .asset
            .ok_or(proto::rpc_store::SyncAccountVaultResponse::missing_field(stringify!(asset)))?
            .try_into()?;
        Ok(word.try_into().unwrap())
    }
}

impl TryFrom<proto::rpc_store::AccountVaultUpdate> for AccountVaultUpdate {
    type Error = RpcError;

    fn try_from(value: proto::rpc_store::AccountVaultUpdate) -> Result<Self, Self::Error> {
        let block_num = value.block_num;

        let asset: Option<Asset> = value.asset.map(TryInto::try_into).transpose()?;

        let vault_key = value
            .vault_key
            .ok_or(proto::rpc_store::SyncAccountVaultResponse::missing_field(stringify!(
                vault_key
            )))?
            .try_into()?;

        Ok(Self {
            block_num: block_num.into(),
            asset,
            vault_key,
        })
    }
}
