use alloc::sync::Arc;
use alloc::vec::Vec;

use crypto::merkle::{InOrderIndex, MmrPeaks, PartialMmr};
use miden_objects::Word;
use miden_objects::block::{BlockHeader, BlockNumber};
use miden_objects::crypto::merkle::{Forest, MerklePath};
use miden_objects::crypto::{self};
use tracing::warn;

use crate::rpc::NodeRpcClient;
use crate::store::StoreError;
use crate::{Client, ClientError};

/// Network information management methods.
impl<AUTH> Client<AUTH> {
    /// Attempts to retrieve the genesis block from the store. If not found,
    /// it requests it from the node and store it.
    pub async fn ensure_genesis_in_place(&mut self) -> Result<BlockHeader, ClientError> {
        let genesis = if let Some((block, _)) = self.store.get_block_header_by_num(0.into()).await?
        {
            block
        } else {
            let genesis = self.retrieve_and_store_genesis().await?;
            self.rpc_api.set_genesis_commitment(genesis.commitment()).await?;
            genesis
        };

        Ok(genesis)
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

    /// Fetches from the store the current view of the chain's [`PartialMmr`].
    pub async fn get_current_partial_mmr(&self) -> Result<PartialMmr, ClientError> {
        self.store.get_current_partial_mmr().await.map_err(Into::into)
    }

    // HELPERS
    // --------------------------------------------------------------------------------------------

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
    let expected_path_len = forest
        .leaf_to_corresponding_tree(block_num.as_usize())
        .expect("forest includes block number") as usize;

    let mut idx = InOrderIndex::from_leaf_pos(block_num.as_usize());
    let mut path_nodes = Vec::with_capacity(expected_path_len);

    for node in merkle_path.nodes().iter().take(expected_path_len) {
        path_nodes.push((idx.sibling(), *node));
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

#[cfg(test)]
mod tests {
    use miden_objects::block::BlockNumber;
    use miden_objects::crypto::merkle::{Forest, InOrderIndex, MerklePath, Mmr, PartialMmr};
    use miden_objects::{Felt, Word};

    use super::adjust_merkle_path_for_forest;

    fn word(n: u64) -> Word {
        Word::new([Felt::new(n), Felt::new(0), Felt::new(0), Felt::new(0)])
    }

    #[test]
    fn adjust_merkle_path_truncates_to_forest_bounds() {
        let forest = Forest::new(5);
        // Forest 5 <=> block 4 is rightmost leaf
        let block_num = BlockNumber::from(4u32);
        let path = MerklePath::new(vec![word(1), word(2), word(3)]);

        let adjusted = adjust_merkle_path_for_forest(&path, block_num, forest);
        // Block 4 conforms a single leaf tree so it should be empty
        assert!(adjusted.is_empty());
    }

    #[test]
    fn adjust_merkle_path_keeps_proof_valid_for_smaller_forest() {
        // Build a proof in a larger forest and ensure truncation does not keep siblings from a
        // different tree in the smaller forest, which would invalidate the proof.
        let mut mmr = Mmr::new();
        for value in 0u64..8 {
            mmr.add(word(value));
        }

        let large_forest = Forest::new(8);
        let small_forest = Forest::new(5);
        let leaf_pos = 4usize;
        let block_num = BlockNumber::from(u32::try_from(leaf_pos).unwrap());

        let proof = mmr.open_at(leaf_pos, large_forest).expect("valid proof");
        let adjusted_nodes =
            adjust_merkle_path_for_forest(&proof.merkle_path, block_num, small_forest);
        let adjusted_path = MerklePath::new(adjusted_nodes.iter().map(|(_, n)| *n).collect());

        let peaks = mmr.peaks_at(small_forest).unwrap();
        let mut partial = PartialMmr::from_peaks(peaks);
        let leaf = mmr.get(leaf_pos).expect("leaf exists");

        partial
            .track(leaf_pos, leaf, &adjusted_path)
            .expect("adjusted path should verify against smaller forest peaks");
    }

    #[test]
    fn adjust_merkle_path_correct_indices() {
        // Forest 6 has trees of size 2 and 4
        let forest = Forest::new(6);
        // Block 1 is on tree with size 4 (merkle path should have 2 nodes)
        let block_num = BlockNumber::from(1u32);
        let nodes = vec![word(10), word(11), word(12), word(13)];
        let path = MerklePath::new(nodes.clone());

        let adjusted = adjust_merkle_path_for_forest(&path, block_num, forest);

        assert_eq!(adjusted.len(), 2);
        assert_eq!(adjusted[0].1, nodes[0]);
        assert_eq!(adjusted[1].1, nodes[1]);

        let mut idx = InOrderIndex::from_leaf_pos(1);
        let expected0 = idx.sibling();
        idx = idx.parent();
        let expected1 = idx.sibling();

        assert_eq!(adjusted[0].0, expected0);
        assert_eq!(adjusted[1].0, expected1);
    }

    #[test]
    fn adjust_path_limit_correct_when_siblings_in_bounds() {
        // Ensure the expected depth limit matters even when the next sibling
        // is "in-bounds" (but not part of the leaf's subtree for that forest)
        let large_leaves = 8usize;
        let small_leaves = 7usize;
        let leaf_pos = 2usize;
        let mut mmr = Mmr::new();
        for value in 0u64..large_leaves as u64 {
            mmr.add(word(value));
        }

        let small_forest = Forest::new(small_leaves);
        let proof = mmr.open_at(leaf_pos, Forest::new(large_leaves)).expect("valid proof");
        let expected_depth =
            small_forest.leaf_to_corresponding_tree(leaf_pos).expect("leaf is in forest") as usize;

        // Confirm the next sibling after the expected depth is still in bounds, which would
        // create an overlong path without the depth cap.
        let mut idx = InOrderIndex::from_leaf_pos(leaf_pos);
        for _ in 0..expected_depth {
            idx = idx.parent();
        }
        let next_sibling = idx.sibling();
        let rightmost = InOrderIndex::from_leaf_pos(small_leaves - 1);
        assert!(next_sibling <= rightmost);
        assert!(proof.merkle_path.depth() as usize > expected_depth);

        let adjusted = adjust_merkle_path_for_forest(
            &proof.merkle_path,
            BlockNumber::from(u32::try_from(leaf_pos).unwrap()),
            small_forest,
        );
        assert_eq!(adjusted.len(), expected_depth);
    }
}
