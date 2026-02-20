use miden_client::block::BlockHeader as NativeBlockHeader;
use crate::prelude::*;

use super::word::Word;

/// Public header for a block, containing commitments to the chain state and the proof attesting to
/// the block's validity.
///
/// Key fields include the previous block commitment, block number, chain/nullifier/note roots,
/// transaction commitments (including the kernel), proof commitment, and a timestamp. Two derived
/// values are exposed:
/// - `sub_commitment`: sequential hash of all fields except the `note_root`.
/// - `commitment`: a 2-to-1 hash of the `sub_commitment` and the `note_root`.
#[bindings]
#[derive(Clone)]
pub struct BlockHeader(NativeBlockHeader);

#[bindings]
impl BlockHeader {
    /// Returns the header version.
    pub fn version(&self) -> u32 {
        self.0.version()
    }

    /// Returns the commitment to the block contents.
    pub fn commitment(&self) -> Word {
        self.0.commitment().into()
    }

    /// Returns the commitment to block metadata.
    #[bindings]
    pub fn sub_commitment(&self) -> Word {
        self.0.sub_commitment().into()
    }

    /// Returns the commitment of the previous block.
    #[bindings]
    pub fn prev_block_commitment(&self) -> Word {
        self.0.prev_block_commitment().into()
    }

    /// Returns the block height.
    pub fn block_num(&self) -> u32 {
        self.0.block_num().as_u32()
    }

    /// Returns the chain commitment.
    #[bindings]
    pub fn chain_commitment(&self) -> Word {
        self.0.chain_commitment().into()
    }

    /// Returns the account root commitment.
    #[bindings]
    pub fn account_root(&self) -> Word {
        self.0.account_root().into()
    }

    /// Returns the nullifier root commitment.
    #[bindings]
    pub fn nullifier_root(&self) -> Word {
        self.0.nullifier_root().into()
    }

    /// Returns the note commitment root.
    #[bindings]
    pub fn note_root(&self) -> Word {
        self.0.note_root().into()
    }

    /// Returns the transaction commitment.
    #[bindings]
    pub fn tx_commitment(&self) -> Word {
        self.0.tx_commitment().into()
    }

    /// Returns the transaction kernel commitment.
    #[bindings]
    pub fn tx_kernel_commitment(&self) -> Word {
        self.0.tx_kernel_commitment().into()
    }

    /// Returns the proof commitment.
    #[bindings]
    pub fn proof_commitment(&self) -> Word {
        self.0.commitment().into()
    }

    /// Returns the block timestamp.
    pub fn timestamp(&self) -> u32 {
        self.0.timestamp()
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeBlockHeader> for BlockHeader {
    fn from(header: NativeBlockHeader) -> Self {
        BlockHeader(header)
    }
}

impl From<&NativeBlockHeader> for BlockHeader {
    fn from(header: &NativeBlockHeader) -> Self {
        BlockHeader(header.clone())
    }
}

impl From<BlockHeader> for NativeBlockHeader {
    fn from(header: BlockHeader) -> Self {
        header.0
    }
}

impl From<&BlockHeader> for NativeBlockHeader {
    fn from(header: &BlockHeader) -> Self {
        header.0.clone()
    }
}
