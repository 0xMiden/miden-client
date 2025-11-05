use miden_client::block::BlockHeader as NativeBlockHeader;
use wasm_bindgen::prelude::*;

use super::word::Word;

/// Wrapper around block header data returned by the network.
#[derive(Clone)]
#[wasm_bindgen]
pub struct BlockHeader(NativeBlockHeader);

#[wasm_bindgen]
impl BlockHeader {
    /// Returns the block version number.
    pub fn version(&self) -> u32 {
        self.0.version()
    }

    /// Returns the overall commitment to the block contents.
    pub fn commitment(&self) -> Word {
        self.0.commitment().into()
    }

    #[wasm_bindgen(js_name = "subCommitment")]
    /// Returns the sub-commitment combining state roots.
    pub fn sub_commitment(&self) -> Word {
        self.0.sub_commitment().into()
    }

    #[wasm_bindgen(js_name = "prevBlockCommitment")]
    /// Returns the commitment of the previous block in the chain.
    pub fn prev_block_commitment(&self) -> Word {
        self.0.prev_block_commitment().into()
    }

    #[wasm_bindgen(js_name = "blockNum")]
    /// Returns the block number.
    pub fn block_num(&self) -> u32 {
        self.0.block_num().as_u32()
    }

    #[wasm_bindgen(js_name = "chainCommitment")]
    /// Returns the chain commitment accumulating historical state.
    pub fn chain_commitment(&self) -> Word {
        self.0.chain_commitment().into()
    }

    #[wasm_bindgen(js_name = "accountRoot")]
    /// Returns the account Merkle root for the block.
    pub fn account_root(&self) -> Word {
        self.0.account_root().into()
    }

    #[wasm_bindgen(js_name = "nullifierRoot")]
    /// Returns the nullifier set root for the block.
    pub fn nullifier_root(&self) -> Word {
        self.0.nullifier_root().into()
    }

    #[wasm_bindgen(js_name = "noteRoot")]
    /// Returns the note commitment root for the block.
    pub fn note_root(&self) -> Word {
        self.0.note_root().into()
    }

    #[wasm_bindgen(js_name = "txCommitment")]
    /// Returns the commitment to the transactions included in the block.
    pub fn tx_commitment(&self) -> Word {
        self.0.tx_commitment().into()
    }

    #[wasm_bindgen(js_name = "txKernelCommitment")]
    /// Returns the commitment to transaction kernels included in the block.
    pub fn tx_kernel_commitment(&self) -> Word {
        self.0.tx_kernel_commitment().into()
    }

    #[wasm_bindgen(js_name = "proofCommitment")]
    /// Returns the proof commitment attesting to block validity.
    pub fn proof_commitment(&self) -> Word {
        self.0.proof_commitment().into()
    }

    /// Returns the timestamp assigned to the block.
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
