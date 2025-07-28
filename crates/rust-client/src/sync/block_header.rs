use alloc::{sync::Arc, vec::Vec};

use crypto::merkle::{InOrderIndex, MmrPeaks, PartialMmr};
use miden_objects::{
    Word,
    block::{BlockHeader, BlockNumber},
    crypto::{
        self,
        merkle::{Forest, MerklePath},
    },
};
use tracing::warn;

use crate::{
    Client, ClientError,
    rpc::NodeRpcClient,
    store::{PartialBlockchainFilter, StoreError},
};

/// Network information management methods.
impl<AUTH> Client<AUTH> {
    /// Attempts to retrieve the genesis block from the store. If not found,
    /// it requests it from the node and store it.
    pub async fn ensure_genesis_in_place(&mut self) -> Result<BlockHeader, ClientError> {
        let genesis = self.store.get_block_header_by_num(0.into()).await?;

        match genesis {
            Some((block, _)) => Ok(block),
            None => self.retrieve_and_store_genesis().await,
        }
    }

    /// Calls `get_block_header_by_number` requesting the genesis block and storing it
    /// in the local database.
    async fn retrieve_and_store_genesis(&mut self) -> Result<BlockHeader, ClientError> {
        let (genesis_block, _) = self
            .rpc_api
            .get_block_header_by_number(Some(BlockNumber::GENESIS), false)
            .await?;

        let blank_mmr_peaks = MmrPeaks::new(Forest::empty(), vec![])
            .expect("Blank MmrPeaks should not fail to instantiate");
        self.store.insert_block_header(&genesis_block, blank_mmr_peaks, false).await?;
        Ok(genesis_block)
    }

    // HELPERS
    // --------------------------------------------------------------------------------------------

    /// Builds the current view of the chain's [`PartialMmr`]. Because we want to add all new
    /// authentication nodes that could come from applying the MMR updates, we need to track all
    /// known leaves thus far.
    pub(crate) async fn build_current_partial_mmr(&self) -> Result<PartialMmr, ClientError> {
        let current_block_num = self.store.get_sync_height().await?;

        let tracked_nodes =
            self.store.get_partial_blockchain_nodes(PartialBlockchainFilter::All).await?;
        let current_peaks =
            self.store.get_partial_blockchain_peaks_by_block_num(current_block_num).await?;

        // FIXME: Because each block stores the peaks for the MMR for the leaf of pos `block_num-1`,
        // we can get an MMR based on those peaks, add the current block number and align it with
        // the set of all nodes in the store.
        // Otherwise, by doing `PartialMmr::from_parts` we would effectively have more nodes than
        // we need for the passed peaks. The alternative here is to truncate the set of all nodes
        // before calling `from_parts`
        //
        // This is a bit hacky but it works. One alternative would be to _just_ get nodes required
        // for tracked blocks in the MMR. This would however block us from the convenience of
        // just getting all nodes from the store.

        let (current_block, has_client_notes) = self
            .store
            .get_block_header_by_num(current_block_num)
            .await?
            .expect("Current block should be in the store");

        let mut current_partial_mmr = PartialMmr::from_peaks(current_peaks);
        let has_client_notes = has_client_notes.into();
        current_partial_mmr.add(current_block.commitment(), has_client_notes);

        let current_partial_mmr =
            PartialMmr::from_parts(current_partial_mmr.peaks(), tracked_nodes, has_client_notes);

        Ok(current_partial_mmr)
    }

    /// Retrieves and stores a [`BlockHeader`] by number, and stores its authentication data as
    /// well.
    ///
    /// If the store already contains MMR data for the requested block number, the request isn't
    /// done and the stored block header is returned.
    pub(crate) async fn get_and_store_authenticated_block(
        &self,
        block_num: BlockNumber,
        current_partial_mmr: &mut PartialMmr,
    ) -> Result<BlockHeader, ClientError> {
        if current_partial_mmr.is_tracked(block_num.as_usize()) {
            warn!("Current partial MMR already contains the requested data");
            let (block_header, _) = self
                .store
                .get_block_header_by_num(block_num)
                .await?
                .expect("Block header should be tracked");
            return Ok(block_header);
        }

        // Fetch the block header and MMR proof from the node
        let (block_header, path_nodes) =
            fetch_block_header(self.rpc_api.clone(), block_num, current_partial_mmr).await?;

        // Insert header and MMR nodes
        self.store
            .insert_block_header(&block_header, current_partial_mmr.peaks(), true)
            .await?;
        self.store.insert_partial_blockchain_nodes(&path_nodes).await?;

        Ok(block_header)
    }
}

// UTILS
// --------------------------------------------------------------------------------------------

/// Returns a merkle path nodes for a specific block adjusted for a defined forest size.
/// This function trims the merkle path to include only the nodes that are relevant for
/// the MMR forest.
///
/// # Parameters
/// - `merkle_path`: Original merkle path.
/// - `block_num`: The block number for which the path is computed.
/// - `forest`: The target size of the forest.
pub(crate) fn adjust_merkle_path_for_forest(
    merkle_path: &MerklePath,
    block_num: BlockNumber,
    forest: Forest,
) -> Vec<(InOrderIndex, Word)> {
    assert!(
        forest.num_leaves() > block_num.as_usize(),
        "Can't adjust merkle path for a forest that does not include the block number"
    );

    let rightmost_index = InOrderIndex::from_leaf_pos(forest.num_leaves() - 1);

    let mut idx = InOrderIndex::from_leaf_pos(block_num.as_usize());
    let mut path_nodes = vec![];
    for node in merkle_path.iter() {
        idx = idx.sibling();
        // Rightmost index is always the biggest value, so if the path contains any node
        // past it, we can discard it for our version of the forest
        if idx <= rightmost_index {
            path_nodes.push((idx, *node));
        }
        idx = idx.parent();
    }

    path_nodes
}

pub(crate) async fn fetch_block_header(
    rpc_api: Arc<dyn NodeRpcClient>,
    block_num: BlockNumber,
    current_partial_mmr: &mut PartialMmr,
) -> Result<(BlockHeader, Vec<(InOrderIndex, Word)>), ClientError> {
    let (block_header, mmr_proof) = rpc_api.get_block_header_with_proof(block_num).await?;

    // Trim merkle path to keep nodes relevant to our current PartialMmr since the node's MMR
    // might be of a forest arbitrarily higher
    let path_nodes = adjust_merkle_path_for_forest(
        &mmr_proof.merkle_path,
        block_num,
        current_partial_mmr.forest(),
    );

    let merkle_path = MerklePath::new(path_nodes.iter().map(|(_, n)| *n).collect());

    current_partial_mmr
        .track(block_num.as_usize(), block_header.commitment(), &merkle_path)
        .map_err(StoreError::MmrError)?;

    Ok((block_header, path_nodes))
}
