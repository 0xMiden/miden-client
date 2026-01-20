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

pub fn process_partial_blockchain_nodes_from_js_value(
    js_value: JsValue,
) -> Result<BTreeMap<InOrderIndex, Word>, StoreError> {
    let partial_blockchain_nodes_idxdb: Vec<PartialBlockchainNodeIdxdbObject> =
        from_value(js_value)
            .map_err(|err| StoreError::DatabaseError(format!("failed to deserialize {err:?}")))?;

    let results: Result<BTreeMap<InOrderIndex, Word>, StoreError> = partial_blockchain_nodes_idxdb
        .into_iter()
        .map(|record| {
            let id_as_u64: u64 = record.id.parse::<u64>().unwrap();
            let id = InOrderIndex::new(
                NonZeroUsize::new(
                    usize::try_from(id_as_u64).expect("usize should not fail converting to u64"),
                )
                .unwrap(),
            );
            let node = Word::try_from(&record.node)?;
            Ok((id, node))
        })
        .collect();

    results
}
