use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use miden_client::Word;
use miden_client::block::BlockHeader;
use miden_client::crypto::{Forest, InOrderIndex, MmrPeaks};
use miden_client::note::BlockNumber;
use miden_client::store::{BlockRelevance, PartialBlockchainFilter, StoreError};
use miden_client::utils::Deserializable;

use super::WebStore;
use crate::promise::{await_js, await_js_value, await_ok};

mod js_bindings;
use js_bindings::{
    idxdb_get_block_headers, idxdb_get_partial_blockchain_nodes,
    idxdb_get_partial_blockchain_nodes_all, idxdb_get_partial_blockchain_peaks_by_block_num,
    idxdb_get_tracked_block_headers, idxdb_insert_block_header,
    idxdb_insert_partial_blockchain_nodes, idxdb_prune_irrelevant_blocks,
};

mod models;
use models::{
    BlockHeaderIdxdbObject, PartialBlockchainNodeIdxdbObject, PartialBlockchainPeaksIdxdbObject,
};

pub mod utils;
use utils::{
    SerializedBlockHeaderData, SerializedPartialBlockchainNodeData,
    process_partial_blockchain_nodes_from_js_value, serialize_block_header,
    serialize_partial_blockchain_node,
};

impl WebStore {
    pub(crate) async fn insert_block_header(
        &self,
        block_header: &BlockHeader,
        partial_blockchain_peaks: MmrPeaks,
        has_client_notes: bool,
    ) -> Result<(), StoreError> {
        let partial_blockchain_peaks = partial_blockchain_peaks.peaks().to_vec();
        let SerializedBlockHeaderData {
            block_num,
            header,
            partial_blockchain_peaks,
            has_client_notes,
        } = serialize_block_header(block_header, &partial_blockchain_peaks, has_client_notes);

        let promise = idxdb_insert_block_header(
            block_num,
            header,
            partial_blockchain_peaks,
            has_client_notes,
        );
        await_ok(promise, "failed to insert block header").await?;

        Ok(())
    }

    pub(crate) async fn get_block_headers(
        &self,
        block_numbers: &BTreeSet<BlockNumber>,
    ) -> Result<Vec<(BlockHeader, BlockRelevance)>, StoreError> {
        let formatted_block_numbers_list: Vec<String> = block_numbers
            .iter()
            .map(|block_number| i64::from(block_number.as_u32()).to_string())
            .collect();

        let promise = idxdb_get_block_headers(formatted_block_numbers_list);
        let block_headers_idxdb: Vec<Option<BlockHeaderIdxdbObject>> =
            await_js(promise, "failed to get block headers").await?;

        // Transform the list of Option<BlockHeaderIdxdbObject> to a list of results
        let results: Result<Vec<(BlockHeader, BlockRelevance)>, StoreError> = block_headers_idxdb
            .into_iter()
            .filter_map(|record_option| record_option.map(Ok))
            .map(|record_result: Result<BlockHeaderIdxdbObject, StoreError>| {
                let record = record_result?;
                let block_header = BlockHeader::read_from_bytes(&record.header)?;
                let has_client_notes = record.has_client_notes.into();

                Ok((block_header, has_client_notes))
            })
            .collect(); // Collects into Result<Vec<(BlockHeader, bool)>, StoreError>

        results
    }

    pub(crate) async fn get_tracked_block_headers(&self) -> Result<Vec<BlockHeader>, StoreError> {
        let promise = idxdb_get_tracked_block_headers();
        let block_headers_idxdb: Vec<BlockHeaderIdxdbObject> =
            await_js(promise, "failed to get tracked block headers").await?;

        let results: Result<Vec<BlockHeader>, StoreError> = block_headers_idxdb
            .into_iter()
            .map(|record| {
                let block_header = BlockHeader::read_from_bytes(&record.header)?;

                Ok(block_header)
            })
            .collect();

        results
    }

    pub(crate) async fn get_partial_blockchain_nodes(
        &self,
        filter: PartialBlockchainFilter,
    ) -> Result<BTreeMap<InOrderIndex, Word>, StoreError> {
        match filter {
            PartialBlockchainFilter::All => {
                let promise = idxdb_get_partial_blockchain_nodes_all();
                let js_value =
                    await_js_value(promise, "failed to get all partial blockchain nodes").await?;
                process_partial_blockchain_nodes_from_js_value(js_value)
            },
            PartialBlockchainFilter::List(ids) => {
                let formatted_list: Vec<String> =
                    ids.iter().map(|id| (Into::<usize>::into(*id)).to_string()).collect();

                let promise = idxdb_get_partial_blockchain_nodes(formatted_list);
                let js_value =
                    await_js_value(promise, "failed to get partial blockchain nodes").await?;
                process_partial_blockchain_nodes_from_js_value(js_value)
            },
        }
    }

    pub(crate) async fn get_partial_blockchain_peaks_by_block_num(
        &self,
        block_num: BlockNumber,
    ) -> Result<MmrPeaks, StoreError> {
        let block_num_as_str = block_num.to_string();

        let promise = idxdb_get_partial_blockchain_peaks_by_block_num(block_num_as_str);
        let mmr_peaks_idxdb: PartialBlockchainPeaksIdxdbObject =
            await_js(promise, "failed to get partial blockchain peaks by block number").await?;

        if let Some(peaks) = mmr_peaks_idxdb.peaks {
            let mmr_peaks_nodes: Vec<Word> = Vec::<Word>::read_from_bytes(&peaks)?;

            return MmrPeaks::new(Forest::new(block_num.as_usize()), mmr_peaks_nodes)
                .map_err(StoreError::MmrError);
        }

        Ok(MmrPeaks::new(Forest::empty(), vec![])?)
    }

    pub(crate) async fn insert_partial_blockchain_nodes(
        &self,
        nodes: &[(InOrderIndex, Word)],
    ) -> Result<(), StoreError> {
        let mut serialized_node_ids = Vec::new();
        let mut serialized_nodes = Vec::new();
        for (id, node) in nodes {
            let SerializedPartialBlockchainNodeData { id, node } =
                serialize_partial_blockchain_node(*id, *node)?;
            serialized_node_ids.push(id);
            serialized_nodes.push(node);
        }

        let promise = idxdb_insert_partial_blockchain_nodes(serialized_node_ids, serialized_nodes);
        await_ok(promise, "failed to insert partial blockchain nodes").await?;

        Ok(())
    }

    pub(crate) async fn prune_irrelevant_blocks(&self) -> Result<(), StoreError> {
        let promise = idxdb_prune_irrelevant_blocks();
        await_ok(promise, "failed to prune block header").await?;

        Ok(())
    }
}
