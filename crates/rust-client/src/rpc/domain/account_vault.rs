use alloc::string::ToString;
use alloc::vec::Vec;

use miden_protocol::Word;
use miden_protocol::asset::{Asset, AssetVaultKey};
use miden_protocol::block::BlockNumber;

use crate::rpc::domain::MissingFieldHelper;
use crate::rpc::{RpcConversionError, RpcError, generated as proto};

// ASSET CONVERSION
// ================================================================================================

impl TryFrom<proto::primitives::Asset> for Asset {
    type Error = RpcConversionError;

    fn try_from(value: proto::primitives::Asset) -> Result<Self, Self::Error> {
        let key_word: Word = value
            .key
            .ok_or(proto::primitives::Asset::missing_field(stringify!(key)))?
            .try_into()?;
        let value_word: Word = value
            .value
            .ok_or(proto::primitives::Asset::missing_field(stringify!(value)))?
            .try_into()?;
        Asset::from_key_value_words(key_word, value_word)
            .map_err(|e| RpcConversionError::InvalidField(e.to_string()))
    }
}

// ACCOUNT VAULT INFO
// ================================================================================================

/// Represents a `proto::rpc::SyncAccountVaultResponse` with fields converted into domain
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

impl TryFrom<proto::rpc::SyncAccountVaultResponse> for AccountVaultInfo {
    type Error = RpcError;

    fn try_from(value: proto::rpc::SyncAccountVaultResponse) -> Result<Self, Self::Error> {
        let pagination_info =
            value
                .pagination_info
                .ok_or(proto::rpc::SyncAccountVaultResponse::missing_field(stringify!(
                    pagination_info
                )))?;
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
    pub vault_key: AssetVaultKey,
}

// ACCOUNT VAULT UPDATE CONVERSION
// ================================================================================================

impl TryFrom<proto::rpc::AccountVaultUpdate> for AccountVaultUpdate {
    type Error = RpcError;

    fn try_from(value: proto::rpc::AccountVaultUpdate) -> Result<Self, Self::Error> {
        let block_num = value.block_num;

        let vault_key_inner: Word = value
            .vault_key
            .ok_or(proto::rpc::SyncAccountVaultResponse::missing_field(stringify!(vault_key)))?
            .try_into()?;
        let vault_key = AssetVaultKey::try_from(vault_key_inner)
            .map_err(|e| RpcError::InvalidResponse(e.to_string()))?;

        let asset = value.asset.map(Asset::try_from).transpose()?;

        if let Some(ref asset) = asset
            && Word::from(asset.vault_key()) != vault_key_inner
        {
            return Err(RpcError::InvalidResponse(
                "account vault update returned mismatched asset key".to_string(),
            ));
        }

        Ok(Self {
            block_num: block_num.into(),
            asset,
            vault_key,
        })
    }
}
