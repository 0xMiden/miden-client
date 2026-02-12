use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::ToString;
use alloc::vec::Vec;
use core::num::NonZeroUsize;

use miden_client::Word;
use miden_client::block::BlockHeader;
use miden_client::crypto::{Forest, InOrderIndex, MmrPeaks};
use miden_client::note::BlockNumber;
use miden_client::store::{BlockRelevance, PartialBlockchainFilter, StoreError};
use miden_client::utils::{Deserializable, Serializable};

use super::WasmSqliteStore;

mod js_bindings;
use js_bindings::{
    js_get_block_headers,
    js_get_partial_blockchain_nodes,
    js_get_partial_blockchain_nodes_all,
    js_get_partial_blockchain_nodes_up_to_inorder_index,
    js_get_partial_blockchain_peaks_by_block_num,
    js_get_tracked_block_headers,
    js_insert_block_header,
    js_insert_partial_blockchain_nodes,
    js_prune_irrelevant_blocks,
};

mod models;
use models::{BlockHeaderObject, PartialBlockchainNodeObject, PartialBlockchainPeaksObject};

impl WasmSqliteStore {
    #[allow(clippy::unused_async)]
    pub(crate) async fn insert_block_header(
        &self,
        block_header: &BlockHeader,
        partial_blockchain_peaks: MmrPeaks,
        has_client_notes: bool,
    ) -> Result<(), StoreError> {
        let block_num = block_header.block_num().as_u32();
        let header = block_header.to_bytes();
        let peaks = partial_blockchain_peaks.peaks().to_bytes();

        js_insert_block_header(self.db_id(), block_num, header, peaks, has_client_notes);
        Ok(())
    }

    #[allow(clippy::unused_async)]
    pub(crate) async fn get_block_headers(
        &self,
        block_numbers: &BTreeSet<BlockNumber>,
    ) -> Result<Vec<(BlockHeader, BlockRelevance)>, StoreError> {
        let block_nums: Vec<u32> = block_numbers.iter().map(BlockNumber::as_u32).collect();
        let js_value = js_get_block_headers(self.db_id(), block_nums);
        let headers: Vec<BlockHeaderObject> =
            serde_wasm_bindgen::from_value(js_value).map_err(|err| {
                StoreError::DatabaseError(format!("failed to deserialize block headers: {err:?}"))
            })?;

        headers
            .into_iter()
            .map(|h| {
                let header = BlockHeader::read_from_bytes(&h.header)?;
                let relevance = BlockRelevance::from(h.has_client_notes);
                Ok((header, relevance))
            })
            .collect::<Result<Vec<_>, StoreError>>()
    }

    #[allow(clippy::unused_async)]
    pub(crate) async fn get_tracked_block_headers(&self) -> Result<Vec<BlockHeader>, StoreError> {
        let js_value = js_get_tracked_block_headers(self.db_id());
        let headers: Vec<BlockHeaderObject> =
            serde_wasm_bindgen::from_value(js_value).map_err(|err| {
                StoreError::DatabaseError(format!(
                    "failed to deserialize tracked block headers: {err:?}"
                ))
            })?;

        headers
            .into_iter()
            .map(|h| BlockHeader::read_from_bytes(&h.header).map_err(Into::into))
            .collect::<Result<Vec<_>, StoreError>>()
    }

    #[allow(clippy::unused_async)]
    pub(crate) async fn get_partial_blockchain_nodes(
        &self,
        filter: PartialBlockchainFilter,
    ) -> Result<BTreeMap<InOrderIndex, Word>, StoreError> {
        let js_value = match filter {
            PartialBlockchainFilter::All => js_get_partial_blockchain_nodes_all(self.db_id()),
            PartialBlockchainFilter::List(indices) => {
                let ids: Vec<String> = indices.iter().map(|idx| idx.inner().to_string()).collect();
                js_get_partial_blockchain_nodes(self.db_id(), ids)
            },
            PartialBlockchainFilter::Forest(forest) => {
                if forest.is_empty() {
                    return Ok(BTreeMap::new());
                }

                let max_index = forest.rightmost_in_order_index();
                js_get_partial_blockchain_nodes_up_to_inorder_index(
                    self.db_id(),
                    max_index.inner().to_string(),
                )
            },
        };

        let nodes: Vec<PartialBlockchainNodeObject> = serde_wasm_bindgen::from_value(js_value)
            .map_err(|err| {
                StoreError::DatabaseError(format!(
                    "failed to deserialize partial blockchain nodes: {err:?}"
                ))
            })?;

        nodes
            .into_iter()
            .map(|n| {
                let id: usize =
                    n.id.parse()
                        .map_err(|e| StoreError::ParsingError(format!("invalid node id: {e}")))?;
                let id = NonZeroUsize::new(id).ok_or_else(|| {
                    StoreError::ParsingError(
                        "partial blockchain node id must be non-zero".to_string(),
                    )
                })?;
                let index = InOrderIndex::new(id);
                let node = Word::read_from_bytes(&n.node)?;
                Ok((index, node))
            })
            .collect::<Result<BTreeMap<InOrderIndex, Word>, StoreError>>()
    }

    #[allow(clippy::unused_async)]
    pub(crate) async fn insert_partial_blockchain_nodes(
        &self,
        nodes: &[(InOrderIndex, Word)],
    ) -> Result<(), StoreError> {
        let ids: Vec<String> = nodes.iter().map(|(idx, _)| idx.inner().to_string()).collect();
        let node_values: Vec<wasm_bindgen::JsValue> = nodes
            .iter()
            .map(|(_, word)| {
                let bytes = word.to_bytes();
                let js_array = js_sys::Uint8Array::from(bytes.as_slice());
                js_array.into()
            })
            .collect();

        js_insert_partial_blockchain_nodes(self.db_id(), ids, node_values);
        Ok(())
    }

    #[allow(clippy::unused_async)]
    pub(crate) async fn get_partial_blockchain_peaks_by_block_num(
        &self,
        block_num: BlockNumber,
    ) -> Result<MmrPeaks, StoreError> {
        let js_value =
            js_get_partial_blockchain_peaks_by_block_num(self.db_id(), block_num.as_u32());
        let peaks_obj: PartialBlockchainPeaksObject = serde_wasm_bindgen::from_value(js_value)
            .map_err(|err| {
                StoreError::DatabaseError(format!("failed to deserialize peaks: {err:?}"))
            })?;

        match peaks_obj.partial_blockchain_peaks {
            Some(peaks_bytes) => {
                let peaks = Vec::<Word>::read_from_bytes(&peaks_bytes)?;
                Ok(MmrPeaks::new(Forest::new(block_num.as_usize()), peaks)?)
            },
            None => Ok(MmrPeaks::new(Forest::empty(), vec![])?),
        }
    }

    #[allow(clippy::unused_async)]
    pub(crate) async fn prune_irrelevant_blocks(&self) -> Result<(), StoreError> {
        js_prune_irrelevant_blocks(self.db_id());
        Ok(())
    }
}
