use alloc::string::ToString;
use alloc::vec::Vec;

use miden_objects::asset::{Asset, VaultKey};
use miden_objects::block::BlockNumber;
use miden_objects::{AssetError, Word};

use crate::rpc::domain::MissingFieldHelper;
use crate::rpc::{RpcError, generated as proto};

// ACCOUNT VAULT INFO
// ================================================================================================

/// Represents a `proto::rpc_store::SyncAccountVaultResponse` with fields converted into domain
/// types. Contains information of asset updates in a given range of blocks specified on request.
/// Also provides the current chain tip while processing the request.
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
            proto::rpc_store::SyncAccountVaultResponse::missing_field(stringify!(pagination_info)),
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

/// Represents an update to an account vault, including the vault key and asset value involved.
pub struct AccountVaultUpdate {
    /// Block number in which the slot was updated.
    pub block_num: BlockNumber,
    /// Asset value related to the vault key. If not present, the asset was removed from the vault.
    pub asset: Option<Asset>,
    /// Vault key associated with the asset.
    pub vault_key: VaultKey,
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
        word.try_into()
            .map_err(|e: AssetError| RpcError::InvalidResponse(e.to_string()))
    }
}

impl TryFrom<proto::rpc_store::AccountVaultUpdate> for AccountVaultUpdate {
    type Error = RpcError;

    fn try_from(value: proto::rpc_store::AccountVaultUpdate) -> Result<Self, Self::Error> {
        let block_num = value.block_num;

        let asset: Option<Asset> = value.asset.map(TryInto::try_into).transpose()?;

        let vault_key_inner: Word = value
            .vault_key
            .ok_or(proto::rpc_store::SyncAccountVaultResponse::missing_field(stringify!(
                vault_key
            )))?
            .try_into()?;
        let vault_key = VaultKey::new_unchecked(vault_key_inner);

        Ok(Self {
            block_num: block_num.into(),
            asset,
            vault_key,
        })
    }
}
