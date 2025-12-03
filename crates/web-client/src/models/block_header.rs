use miden_client::block::BlockHeader as NativeBlockHeader;
use wasm_bindgen::prelude::*;

use super::word::Word;

/// Public header data for a block.
#[derive(Clone)]
#[wasm_bindgen]
pub struct BlockHeader(NativeBlockHeader);

#[wasm_bindgen]
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
    #[wasm_bindgen(js_name = "subCommitment")]
    pub fn sub_commitment(&self) -> Word {
        self.0.sub_commitment().into()
    }

    /// Returns the commitment of the previous block.
    #[wasm_bindgen(js_name = "prevBlockCommitment")]
    pub fn prev_block_commitment(&self) -> Word {
        self.0.prev_block_commitment().into()
    }

    /// Returns the block height.
    #[wasm_bindgen(js_name = "blockNum")]
    pub fn block_num(&self) -> u32 {
        self.0.block_num().as_u32()
    }

    /// Returns the chain commitment.
    #[wasm_bindgen(js_name = "chainCommitment")]
    pub fn chain_commitment(&self) -> Word {
        self.0.chain_commitment().into()
    }

    /// Returns the account root commitment.
    #[wasm_bindgen(js_name = "accountRoot")]
    pub fn account_root(&self) -> Word {
        self.0.account_root().into()
    }

    /// Returns the nullifier root commitment.
    #[wasm_bindgen(js_name = "nullifierRoot")]
    pub fn nullifier_root(&self) -> Word {
        self.0.nullifier_root().into()
    }

    /// Returns the note commitment root.
    #[wasm_bindgen(js_name = "noteRoot")]
    pub fn note_root(&self) -> Word {
        self.0.note_root().into()
    }

    /// Returns the transaction commitment.
    #[wasm_bindgen(js_name = "txCommitment")]
    pub fn tx_commitment(&self) -> Word {
        self.0.tx_commitment().into()
    }

    /// Returns the transaction kernel commitment.
    #[wasm_bindgen(js_name = "txKernelCommitment")]
    pub fn tx_kernel_commitment(&self) -> Word {
        self.0.tx_kernel_commitment().into()
    }

    /// Returns the proof commitment.
    #[wasm_bindgen(js_name = "proofCommitment")]
    pub fn proof_commitment(&self) -> Word {
        self.0.proof_commitment().into()
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
