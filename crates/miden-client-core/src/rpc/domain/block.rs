use miden_objects::block::{BlockHeader, BlockNumber, FeeParameters};

use crate::rpc::domain::MissingFieldHelper;
use crate::rpc::errors::RpcConversionError;
use crate::rpc::generated as proto;

// BLOCK HEADER
// ================================================================================================

impl From<&BlockHeader> for proto::blockchain::BlockHeader {
    fn from(header: &BlockHeader) -> Self {
        Self {
            version: header.version(),
            prev_block_commitment: Some(header.prev_block_commitment().into()),
            block_num: header.block_num().as_u32(),
            chain_commitment: Some(header.chain_commitment().into()),
            account_root: Some(header.account_root().into()),
            nullifier_root: Some(header.nullifier_root().into()),
            note_root: Some(header.note_root().into()),
            tx_commitment: Some(header.tx_commitment().into()),
            tx_kernel_commitment: Some(header.tx_kernel_commitment().into()),
            proof_commitment: Some(header.proof_commitment().into()),
            timestamp: header.timestamp(),
            fee_parameters: Some(header.fee_parameters().into()),
        }
    }
}

impl From<&FeeParameters> for proto::blockchain::FeeParameters {
    fn from(fee_params: &FeeParameters) -> Self {
        Self {
            native_asset_id: Some(fee_params.native_asset_id().into()),
            verification_base_fee: fee_params.verification_base_fee(),
        }
    }
}

impl From<FeeParameters> for proto::blockchain::FeeParameters {
    fn from(fee_params: FeeParameters) -> Self {
        (&fee_params).into()
    }
}

impl From<BlockHeader> for proto::blockchain::BlockHeader {
    fn from(header: BlockHeader) -> Self {
        (&header).into()
    }
}

impl TryFrom<proto::blockchain::BlockHeader> for BlockHeader {
    type Error = RpcConversionError;

    fn try_from(value: proto::blockchain::BlockHeader) -> Result<Self, Self::Error> {
        Ok(BlockHeader::new(
            value.version,
            value
                .prev_block_commitment
                .ok_or(proto::blockchain::BlockHeader::missing_field(stringify!(
                    prev_block_commitment
                )))?
                .try_into()?,
            value.block_num.into(),
            value
                .chain_commitment
                .ok_or(proto::blockchain::BlockHeader::missing_field(stringify!(chain_commitment)))?
                .try_into()?,
            value
                .account_root
                .ok_or(proto::blockchain::BlockHeader::missing_field(stringify!(account_root)))?
                .try_into()?,
            value
                .nullifier_root
                .ok_or(proto::blockchain::BlockHeader::missing_field(stringify!(nullifier_root)))?
                .try_into()?,
            value
                .note_root
                .ok_or(proto::blockchain::BlockHeader::missing_field(stringify!(note_root)))?
                .try_into()?,
            value
                .tx_commitment
                .ok_or(proto::blockchain::BlockHeader::missing_field(stringify!(tx_commitment)))?
                .try_into()?,
            value
                .tx_kernel_commitment
                .ok_or(proto::blockchain::BlockHeader::missing_field(stringify!(
                    tx_kernel_commitment
                )))?
                .try_into()?,
            value
                .proof_commitment
                .ok_or(proto::blockchain::BlockHeader::missing_field(stringify!(proof_commitment)))?
                .try_into()?,
            value
                .fee_parameters
                .ok_or(proto::blockchain::BlockHeader::missing_field(stringify!(fee_parameters)))?
                .try_into()?,
            value.timestamp,
        ))
    }
}

impl TryFrom<&proto::blockchain::FeeParameters> for FeeParameters {
    type Error = RpcConversionError;

    fn try_from(value: &proto::blockchain::FeeParameters) -> Result<Self, Self::Error> {
        let account_id = value
            .native_asset_id
            .clone()
            .ok_or(proto::blockchain::FeeParameters::missing_field("account_id"))?
            .try_into()?;

        FeeParameters::new(account_id, value.verification_base_fee)
            .map_err(|_err| RpcConversionError::InvalidField(stringify!(FeeParameter).into()))
    }
}

impl TryFrom<proto::blockchain::FeeParameters> for FeeParameters {
    type Error = RpcConversionError;

    fn try_from(value: proto::blockchain::FeeParameters) -> Result<Self, Self::Error> {
        FeeParameters::try_from(&value)
    }
}

// BLOCK NUMBER
// ================================================================================================

impl From<BlockNumber> for proto::blockchain::BlockNumber {
    fn from(value: BlockNumber) -> Self {
        Self { block_num: value.as_u32() }
    }
}
