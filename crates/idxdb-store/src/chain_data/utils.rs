use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::num::NonZeroUsize;

use miden_client::Word;
use miden_client::block::BlockHeader;
use miden_client::crypto::InOrderIndex;
use miden_client::store::StoreError;
use miden_client::utils::Serializable;
use serde_wasm_bindgen::from_value;
use wasm_bindgen::JsValue;

use crate::chain_data::PartialBlockchainNodeIdxdbObject;

pub struct SerializedBlockHeaderData {
    pub block_num: u32,
    pub header: Vec<u8>,
    pub partial_blockchain_peaks: Vec<u8>,
    pub has_client_notes: bool,
}

pub struct SerializedPartialBlockchainNodeData {
    pub id: String,
    pub node: String,
}

pub fn serialize_block_header(
    block_header: &BlockHeader,
    partial_blockchain_peaks: &[Word],
    has_client_notes: bool,
) -> SerializedBlockHeaderData {
    let block_num = block_header.block_num().as_u32();
    let header = block_header.to_bytes();
    let partial_blockchain_peaks = partial_blockchain_peaks.to_bytes();

    SerializedBlockHeaderData {
        block_num,
        header,
        partial_blockchain_peaks,
        has_client_notes,
    }
}

pub fn serialize_partial_blockchain_node(
    id: InOrderIndex,
    node: Word,
) -> Result<SerializedPartialBlockchainNodeData, StoreError> {
    let id: u64 = id.inner().try_into()?;
    let id_as_str = id.to_string();
    let node = node.to_string();
    Ok(SerializedPartialBlockchainNodeData { id: id_as_str, node })
}

/// Deserializes partial blockchain nodes from a JS value.
///
/// Note: In WASM, `usize` is 32 bits, which limits the maximum representable `InOrderIndex`.
/// For an MMR with N blocks, the rightmost in-order index is `2N - 1`. To fit in 32 bits:
/// `2N - 1 ≤ u32::MAX` → `N ≤ 2^31` (~2.15 billion blocks).
///
/// This means WASM clients can only support blockchains with up to ~2^31 blocks.
/// Supporting the full `u32::MAX` blocks would require `InOrderIndex` in `miden-crypto`
/// to use `u64` instead of `usize`.
pub fn process_partial_blockchain_nodes_from_js_value(
    js_value: JsValue,
) -> Result<BTreeMap<InOrderIndex, Word>, StoreError> {
    let partial_blockchain_nodes_idxdb: Vec<PartialBlockchainNodeIdxdbObject> =
        from_value(js_value)
            .map_err(|err| StoreError::DatabaseError(format!("failed to deserialize {err:?}")))?;

    let results: Result<BTreeMap<InOrderIndex, Word>, StoreError> = partial_blockchain_nodes_idxdb
        .into_iter()
        .map(|record| {
            let raw_id = record.id;
            // NOTE: This conversion can fail in WASM for blockchains with more than 2^31 blocks.
            // See function docs for details.
            let id_u64: u64 = raw_id.parse().map_err(|_| {
                StoreError::ParsingError(format!(
                    "partial blockchain node id {raw_id} is not a valid number"
                ))
            })?;
            let id = usize::try_from(id_u64).map_err(|_| {
                StoreError::ParsingError(format!(
                    "partial blockchain node id {raw_id} exceeds usize capacity"
                ))
            })?;
            let id = NonZeroUsize::new(id).ok_or_else(|| {
                StoreError::ParsingError("partial blockchain node id must be non-zero".to_string())
            })?;
            let id = InOrderIndex::new(id);
            let node = Word::try_from(&record.node)?;
            Ok((id, node))
        })
        .collect();

    results
}
