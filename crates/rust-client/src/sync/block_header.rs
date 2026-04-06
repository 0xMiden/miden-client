use alloc::sync::Arc;
use alloc::vec::Vec;

use miden_protocol::Word;
use miden_protocol::account::AccountId;
use miden_protocol::block::account_tree::AccountTree;
use miden_protocol::block::nullifier_tree::NullifierTree;
use miden_protocol::block::{BlockHeader, BlockNoteTree, BlockNumber, Blockchain, FeeParameters};
use miden_protocol::crypto::dsa::ecdsa_k256_keccak::SecretKey;
use miden_protocol::crypto::merkle::MerklePath;
use miden_protocol::crypto::merkle::mmr::{Forest, InOrderIndex, MmrPeaks, PartialMmr};
use miden_protocol::crypto::merkle::smt::Smt;
use miden_protocol::transaction::{OrderedTransactionHeaders, TransactionKernel};
use tracing::warn;

use crate::rpc::NodeRpcClient;
use crate::store::{BlockRelevance, StoreError};
use crate::{Client, ClientError};

/// Synthetic faucet account ID used only by the offline bootstrap genesis header so fee
/// parameters can reference a stable native-asset faucet without requiring node state.
const OFFLINE_NATIVE_ASSET_FAUCET_ID: u128 = 0xab00_0000_0000_cd20_0000_ac00_0000_de00;

/// Network information management methods.
impl<AUTH> Client<AUTH> {
    /// Retrieves a block header by its block number from the store.
    ///
    /// Returns `None` if the block header is not found in the store.
    pub async fn get_block_header_by_num(
        &self,
        block_num: BlockNumber,
    ) -> Result<Option<(BlockHeader, BlockRelevance)>, ClientError> {
        self.store.get_block_header_by_num(block_num).await.map_err(Into::into)
    }

    /// Ensures that the genesis block is available. If the genesis commitment is already
    /// cached in the RPC client, returns early. Otherwise, fetches the genesis block from
    /// the node, stores it, and sets the commitment in the RPC client.
    pub async fn ensure_genesis_in_place(&mut self) -> Result<(), ClientError> {
        if self.rpc_api.has_genesis_commitment().is_some() {
            return Ok(());
        }

        let (genesis, _) = self
            .rpc_api
            .get_block_header_by_number(Some(BlockNumber::GENESIS), false)
            .await?;

        let blank_mmr_peaks = MmrPeaks::new(Forest::empty(), vec![])
            .expect("Blank MmrPeaks should not fail to instantiate");
        self.store.insert_block_header(&genesis, blank_mmr_peaks, false).await?;
        self.rpc_api.set_genesis_commitment(genesis.commitment()).await?;
        Ok(())
    }

    /// Seeds the local client state needed to create accounts and execute programs without a node.
    ///
    /// This stores default RPC limits and inserts a synthetic genesis header if one is not
    /// already present in the store. The synthetic header is only intended for local-only
    /// execution and debugging.
    pub async fn prepare_offline_bootstrap(&mut self) -> Result<(), ClientError> {
        let limits = self.store.get_rpc_limits().await?.unwrap_or_default();
        self.store.set_rpc_limits(limits).await?;
        self.rpc_api.set_rpc_limits(limits).await;

        if let Some((genesis, _)) = self.store.get_block_header_by_num(BlockNumber::GENESIS).await?
        {
            self.rpc_api.set_genesis_commitment(genesis.commitment()).await?;
            return Ok(());
        }

        let genesis = synthetic_offline_genesis_header();
        let blank_mmr_peaks = MmrPeaks::new(Forest::empty(), vec![])
            .expect("Blank MmrPeaks should not fail to instantiate");
        self.store.insert_block_header(&genesis, blank_mmr_peaks, false).await?;
        self.rpc_api.set_genesis_commitment(genesis.commitment()).await?;
        Ok(())
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
        let tracked_nodes = authenticated_block_nodes(&block_header, path_nodes);

        // Insert header and MMR nodes
        self.store
            .insert_block_header(&block_header, current_partial_mmr.peaks(), true)
            .await?;
        self.store.insert_partial_blockchain_nodes(&tracked_nodes).await?;

        Ok(block_header)
    }
}

fn synthetic_offline_genesis_header() -> BlockHeader {
    let native_asset_id = AccountId::try_from(OFFLINE_NATIVE_ASSET_FAUCET_ID)
        .expect("offline native asset faucet ID should be valid");
    let fee_parameters =
        FeeParameters::new(native_asset_id, 500).expect("offline fee params should be valid");
    let validator_key = SecretKey::with_rng(&mut rand::rng()).public_key();
    let transactions = OrderedTransactionHeaders::new_unchecked(Vec::new());

    BlockHeader::new(
        0,
        Word::empty(),
        BlockNumber::GENESIS,
        Blockchain::new().commitment(),
        AccountTree::<Smt>::default().root(),
        NullifierTree::<Smt>::default().root(),
        BlockNoteTree::empty().root(),
        transactions.commitment(),
        TransactionKernel.to_commitment(),
        validator_key,
        fee_parameters,
        0,
    )
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

fn authenticated_block_nodes(
    block_header: &BlockHeader,
    mut path_nodes: Vec<(InOrderIndex, Word)>,
) -> Vec<(InOrderIndex, Word)> {
    let mut nodes = Vec::with_capacity(path_nodes.len() + 1);
    nodes.push((
        InOrderIndex::from_leaf_pos(block_header.block_num().as_usize()),
        block_header.commitment(),
    ));
    nodes.append(&mut path_nodes);
    nodes
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
        mmr_proof.merkle_path(),
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
    use alloc::vec::Vec;

    use miden_protocol::block::account_tree::AccountTree;
    use miden_protocol::block::nullifier_tree::NullifierTree;
    use miden_protocol::block::{BlockHeader, BlockNoteTree, BlockNumber, Blockchain};
    use miden_protocol::crypto::merkle::MerklePath;
    use miden_protocol::crypto::merkle::mmr::{Forest, InOrderIndex, Mmr, PartialMmr};
    use miden_protocol::crypto::merkle::smt::Smt;
    use miden_protocol::transaction::{OrderedTransactionHeaders, TransactionKernel};
    use miden_protocol::{Felt, Word};

    use super::{
        adjust_merkle_path_for_forest,
        authenticated_block_nodes,
        synthetic_offline_genesis_header,
    };

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
            adjust_merkle_path_for_forest(proof.merkle_path(), block_num, small_forest);
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
        assert!(proof.merkle_path().depth() as usize > expected_depth);

        let adjusted = adjust_merkle_path_for_forest(
            proof.merkle_path(),
            BlockNumber::from(u32::try_from(leaf_pos).unwrap()),
            small_forest,
        );
        assert_eq!(adjusted.len(), expected_depth);
    }

    #[test]
    fn authenticated_block_nodes_include_leaf_commitment() {
        let block_header = BlockHeader::mock(4, None, None, &[], TransactionKernel.to_commitment());
        let path_nodes = vec![
            (InOrderIndex::from_leaf_pos(4).sibling(), word(10)),
            (InOrderIndex::from_leaf_pos(4).parent().sibling(), word(11)),
        ];

        let nodes = authenticated_block_nodes(&block_header, path_nodes.clone());

        assert_eq!(nodes[0], (InOrderIndex::from_leaf_pos(4), block_header.commitment()));
        assert_eq!(&nodes[1..], path_nodes.as_slice());
    }

    #[test]
    fn synthetic_offline_genesis_header_matches_empty_chain_state() {
        let genesis = synthetic_offline_genesis_header();

        assert_eq!(genesis.block_num(), BlockNumber::GENESIS);
        assert_eq!(genesis.prev_block_commitment(), Word::empty());
        assert_eq!(genesis.chain_commitment(), Blockchain::new().commitment());
        assert_eq!(genesis.account_root(), AccountTree::<Smt>::default().root());
        assert_eq!(genesis.nullifier_root(), NullifierTree::<Smt>::default().root());
        assert_eq!(genesis.note_root(), BlockNoteTree::empty().root());
        assert_eq!(
            genesis.tx_commitment(),
            OrderedTransactionHeaders::new_unchecked(Vec::new()).commitment()
        );
        assert_eq!(genesis.tx_kernel_commitment(), TransactionKernel.to_commitment());
    }
}
