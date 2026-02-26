#![allow(clippy::items_after_statements)]

use std::collections::{BTreeMap, BTreeSet};
use std::num::NonZeroUsize;
use std::rc::Rc;
use std::vec::Vec;

use miden_client::Word;
use miden_client::block::BlockHeader;
use miden_client::crypto::{Forest, InOrderIndex, MmrPeaks};
use miden_client::note::BlockNumber;
use miden_client::store::{BlockRelevance, PartialBlockchainFilter, StoreError};
use miden_client::utils::{Deserializable, Serializable};
use rusqlite::types::Value;
use rusqlite::{Connection, OptionalExtension, Transaction, params, params_from_iter};

use super::SqliteStore;
use crate::sql_error::SqlResultExt;
use crate::{insert_sql, subst};

struct SerializedBlockHeaderData {
    block_num: u32,
    header: Vec<u8>,
    partial_blockchain_peaks: Vec<u8>,
    has_client_notes: bool,
}
struct SerializedBlockHeaderParts {
    _block_num: u64,
    header: Vec<u8>,
    _partial_blockchain_peaks: Vec<u8>,
    has_client_notes: bool,
}

struct SerializedPartialBlockchainNodeData {
    id: i64,
    node: String,
}
struct SerializedPartialBlockchainNodeParts {
    id: u64,
    node: String,
}

impl SqliteStore {
    pub(crate) fn insert_block_header(
        conn: &mut Connection,
        block_header: &BlockHeader,
        partial_blockchain_peaks: &MmrPeaks,
        has_client_notes: bool,
    ) -> Result<(), StoreError> {
        let tx = conn.transaction().into_store_error()?;

        Self::insert_block_header_tx(
            &tx,
            block_header,
            partial_blockchain_peaks,
            has_client_notes,
        )?;

        tx.commit().into_store_error()?;
        Ok(())
    }

    pub(crate) fn get_block_headers(
        conn: &mut Connection,
        block_numbers: &BTreeSet<BlockNumber>,
    ) -> Result<Vec<(BlockHeader, BlockRelevance)>, StoreError> {
        let block_number_list = block_numbers
            .iter()
            .map(|block_number| Value::Integer(i64::from(block_number.as_u32())))
            .collect::<Vec<Value>>();

        const QUERY: &str = "SELECT block_num, header, partial_blockchain_peaks, has_client_notes FROM block_headers WHERE block_num IN rarray(?)";

        conn.prepare(QUERY)
            .into_store_error()?
            .query_map(params![Rc::new(block_number_list)], parse_block_headers_columns)
            .into_store_error()?
            .map(|result| {
                let serialized_block_header_parts: SerializedBlockHeaderParts =
                    result.into_store_error()?;
                parse_block_header(&serialized_block_header_parts)
            })
            .collect()
    }

    pub(crate) fn get_tracked_block_headers(
        conn: &mut Connection,
    ) -> Result<Vec<BlockHeader>, StoreError> {
        const QUERY: &str = "SELECT block_num, header, partial_blockchain_peaks, has_client_notes FROM block_headers WHERE has_client_notes=true";
        conn.prepare(QUERY)
            .into_store_error()?
            .query_map(params![], parse_block_headers_columns)
            .into_store_error()?
            .map(|result| {
                let serialized_block_header_parts: SerializedBlockHeaderParts =
                    result.into_store_error()?;
                parse_block_header(&serialized_block_header_parts).map(|(block, _)| block)
            })
            .collect()
    }

    pub(crate) fn get_partial_blockchain_nodes(
        conn: &mut Connection,
        filter: &PartialBlockchainFilter,
    ) -> Result<BTreeMap<InOrderIndex, Word>, StoreError> {
        match filter {
            PartialBlockchainFilter::All => query_partial_blockchain_nodes(
                conn,
                "SELECT id, node FROM partial_blockchain_nodes",
                params![],
            ),

            PartialBlockchainFilter::List(ids) if ids.is_empty() => Ok(BTreeMap::new()),
            PartialBlockchainFilter::List(ids) => {
                let id_values = ids
                    .iter()
                    .map(|id| Value::Integer(i64::try_from(id.inner()).expect("id is a valid i64")))
                    .collect::<Vec<_>>();

                query_partial_blockchain_nodes(
                    conn,
                    "SELECT id, node FROM partial_blockchain_nodes WHERE id IN rarray(?)",
                    params_from_iter([Rc::new(id_values)]),
                )
            },

            PartialBlockchainFilter::Forest(forest) if forest.is_empty() => Ok(BTreeMap::new()),
            PartialBlockchainFilter::Forest(forest) => {
                let max_index = i64::try_from(forest.rightmost_in_order_index().inner())
                    .expect("id is a valid i64");

                query_partial_blockchain_nodes(
                    conn,
                    "SELECT id, node FROM partial_blockchain_nodes WHERE id <= ?",
                    params![max_index],
                )
            },
        }
    }

    pub(crate) fn get_partial_blockchain_peaks_by_block_num(
        conn: &mut Connection,
        block_num: BlockNumber,
    ) -> Result<MmrPeaks, StoreError> {
        const QUERY: &str =
            "SELECT partial_blockchain_peaks FROM block_headers WHERE block_num = ?";

        let partial_blockchain_peaks: Option<Vec<u8>> = conn
            .prepare(QUERY)
            .into_store_error()?
            .query_row(params![block_num.as_u32()], |row| row.get::<_, Vec<u8>>(0))
            .optional()
            .into_store_error()?;

        if let Some(partial_blockchain_peaks) = partial_blockchain_peaks {
            return parse_partial_blockchain_peaks(block_num.as_u32(), &partial_blockchain_peaks);
        }

        Ok(MmrPeaks::new(Forest::empty(), vec![])?)
    }

    pub fn insert_partial_blockchain_nodes(
        conn: &mut Connection,
        nodes: &[(InOrderIndex, Word)],
    ) -> Result<(), StoreError> {
        let tx = conn.transaction().into_store_error()?;

        Self::insert_partial_blockchain_nodes_tx(&tx, nodes)?;
        tx.commit().into_store_error()?;
        Ok(())
    }

    /// Inserts a list of MMR authentication nodes to the Partial Blockchain nodes table.
    pub(crate) fn insert_partial_blockchain_nodes_tx(
        tx: &Transaction<'_>,
        nodes: &[(InOrderIndex, Word)],
    ) -> Result<(), StoreError> {
        for (index, node) in nodes {
            insert_partial_blockchain_node(tx, *index, *node)?;
        }
        Ok(())
    }

    /// Inserts a block header using a [`rusqlite::Transaction`].
    ///
    /// If the block header exists and `has_client_notes` is `true` then the `has_client_notes`
    /// column is updated to `true` to signify that the block now contains a relevant note.
    pub(crate) fn insert_block_header_tx(
        tx: &Transaction<'_>,
        block_header: &BlockHeader,
        partial_blockchain_peaks: &MmrPeaks,
        has_client_notes: bool,
    ) -> Result<(), StoreError> {
        let partial_blockchain_peaks = partial_blockchain_peaks.peaks().to_vec();
        let SerializedBlockHeaderData {
            block_num,
            header,
            partial_blockchain_peaks,
            has_client_notes,
        } = serialize_block_header(block_header, &partial_blockchain_peaks, has_client_notes);
        const QUERY: &str = insert_sql!(
            block_headers {
                block_num,
                header,
                partial_blockchain_peaks,
                has_client_notes,
            } | IGNORE
        );
        tx.execute(QUERY, params![block_num, header, partial_blockchain_peaks, has_client_notes])
            .into_store_error()?;

        set_block_header_has_client_notes(tx, u64::from(block_num), has_client_notes)?;
        Ok(())
    }

    /// Removes block headers that do not contain any client notes and aren't the genesis or last
    /// block.
    pub fn prune_irrelevant_blocks(conn: &mut Connection) -> Result<(), StoreError> {
        let tx = conn.transaction().into_store_error()?;
        let genesis: u32 = BlockNumber::GENESIS.as_u32();

        let sync_block: Option<u32> = tx
            .query_row("SELECT block_num FROM state_sync LIMIT 1", [], |r| r.get(0))
            .optional()
            .into_store_error()?;

        if let Some(sync_height) = sync_block {
            tx.execute(
                r"
            DELETE FROM block_headers
            WHERE has_client_notes = 0
              AND block_num > ?1
              AND block_num < ?2
            ",
                rusqlite::params![genesis, sync_height],
            )
            .into_store_error()?;
        }

        tx.commit().into_store_error()
    }
}

// HELPERS
// ================================================================================================

/// Inserts a node represented by its in-order index and the node value.
fn insert_partial_blockchain_node(
    tx: &Transaction<'_>,
    id: InOrderIndex,
    node: Word,
) -> Result<(), StoreError> {
    let SerializedPartialBlockchainNodeData { id, node } =
        serialize_partial_blockchain_node(id, node);
    const QUERY: &str = insert_sql!(partial_blockchain_nodes { id, node } | IGNORE);
    tx.execute(QUERY, params![id, node]).into_store_error()?;
    Ok(())
}

fn query_partial_blockchain_nodes<P: rusqlite::Params>(
    conn: &mut Connection,
    sql: &str,
    params: P,
) -> Result<BTreeMap<InOrderIndex, Word>, StoreError> {
    let mut stmt = conn.prepare_cached(sql).into_store_error()?;

    stmt.query_map(params, parse_partial_blockchain_nodes_columns)
        .into_store_error()?
        .map(|row_res| {
            let parts: SerializedPartialBlockchainNodeParts = row_res.into_store_error()?;
            parse_partial_blockchain_nodes(&parts)
        })
        .collect()
}

fn parse_partial_blockchain_peaks(forest: u32, peaks_nodes: &[u8]) -> Result<MmrPeaks, StoreError> {
    let mmr_peaks_nodes = Vec::<Word>::read_from_bytes(peaks_nodes)?;

    MmrPeaks::new(
        Forest::new(usize::try_from(forest).map_err(|_| {
            StoreError::ParsingError("forest value exceeds usize".to_string())
        })?),
        mmr_peaks_nodes,
    )
    .map_err(StoreError::MmrError)
}

fn serialize_block_header(
    block_header: &BlockHeader,
    partial_blockchain_peaks: &[Word],
    has_client_notes: bool,
) -> SerializedBlockHeaderData {
    let block_num = block_header.block_num();
    let header = block_header.to_bytes();
    let partial_blockchain_peaks = partial_blockchain_peaks.to_bytes();

    SerializedBlockHeaderData {
        block_num: block_num.as_u32(),
        header,
        partial_blockchain_peaks,
        has_client_notes,
    }
}

fn parse_block_headers_columns(
    row: &rusqlite::Row<'_>,
) -> Result<SerializedBlockHeaderParts, rusqlite::Error> {
    let block_num: u32 = row.get(0)?;
    let header: Vec<u8> = row.get(1)?;
    let partial_blockchain_peaks: Vec<u8> = row.get(2)?;
    let has_client_notes: bool = row.get(3)?;

    Ok(SerializedBlockHeaderParts {
        _block_num: u64::from(block_num),
        header,
        _partial_blockchain_peaks: partial_blockchain_peaks,
        has_client_notes,
    })
}

fn parse_block_header(
    serialized_block_header_parts: &SerializedBlockHeaderParts,
) -> Result<(BlockHeader, BlockRelevance), StoreError> {
    Ok((
        BlockHeader::read_from_bytes(&serialized_block_header_parts.header)?,
        serialized_block_header_parts.has_client_notes.into(),
    ))
}

fn serialize_partial_blockchain_node(
    id: InOrderIndex,
    node: Word,
) -> SerializedPartialBlockchainNodeData {
    let id = i64::try_from(id.inner()).expect("id is a valid i64");
    let node = node.to_hex();
    SerializedPartialBlockchainNodeData { id, node }
}

fn parse_partial_blockchain_nodes_columns(
    row: &rusqlite::Row<'_>,
) -> Result<SerializedPartialBlockchainNodeParts, rusqlite::Error> {
    let id: u64 = row.get(0)?;
    let node = row.get(1)?;
    Ok(SerializedPartialBlockchainNodeParts { id, node })
}

fn parse_partial_blockchain_nodes(
    serialized_partial_blockchain_node_parts: &SerializedPartialBlockchainNodeParts,
) -> Result<(InOrderIndex, Word), StoreError> {
    let id = InOrderIndex::new(
        NonZeroUsize::new(
            usize::try_from(serialized_partial_blockchain_node_parts.id)
                .expect("id is u64, should not fail"),
        )
        .unwrap(),
    );
    let node: Word = Word::try_from(&serialized_partial_blockchain_node_parts.node)?;
    Ok((id, node))
}

pub(crate) fn set_block_header_has_client_notes(
    tx: &Transaction<'_>,
    block_num: u64,
    has_client_notes: bool,
) -> Result<(), StoreError> {
    // Only update to change has_client_notes to true if it was false previously
    const QUERY: &str = "\
        UPDATE block_headers
        SET has_client_notes=?
        WHERE block_num=? AND has_client_notes=FALSE;";
    tx.execute(QUERY, params![has_client_notes, block_num]).into_store_error()?;
    Ok(())
}

#[cfg(test)]
mod test {
    use std::collections::{BTreeMap, BTreeSet};
    use std::vec::Vec;

    use miden_client::Word;
    use miden_client::block::BlockHeader;
    use miden_client::crypto::{Forest, InOrderIndex, MmrPeaks};
    use miden_client::store::Store;
    use miden_protocol::crypto::merkle::mmr::Mmr;
    use miden_protocol::transaction::TransactionKernel;
    use rusqlite::params;

    use crate::SqliteStore;
    use crate::tests::create_test_store;

    async fn insert_dummy_block_headers(store: &mut SqliteStore) -> Vec<BlockHeader> {
        let block_headers: Vec<BlockHeader> = (0..5)
            .map(|block_num| {
                BlockHeader::mock(block_num, None, None, &[], TransactionKernel.to_commitment())
            })
            .collect();

        let block_headers_clone = block_headers.clone();
        store
            .interact_with_connection(move |conn| {
                let tx = conn.transaction().unwrap();
                let dummy_peaks = MmrPeaks::new(Forest::empty(), Vec::new()).unwrap();
                (0..5).for_each(|block_num| {
                    SqliteStore::insert_block_header_tx(
                        &tx,
                        &block_headers_clone[block_num],
                        &dummy_peaks,
                        false,
                    )
                    .unwrap();
                });
                tx.commit().unwrap();
                Ok(())
            })
            .await
            .unwrap();

        block_headers
    }

    #[tokio::test]
    async fn insert_and_get_block_headers_by_number() {
        let mut store = create_test_store().await;
        let block_headers = insert_dummy_block_headers(&mut store).await;

        let block_header = Store::get_block_header_by_num(&store, 3.into()).await.unwrap().unwrap();
        assert_eq!(block_headers[3], block_header.0);
    }

    #[tokio::test]
    async fn insert_and_get_block_headers_by_list() {
        let mut store = create_test_store().await;
        let mock_block_headers = insert_dummy_block_headers(&mut store).await;

        let block_headers: Vec<BlockHeader> =
            Store::get_block_headers(&store, &[1.into(), 3.into()].into_iter().collect())
                .await
                .unwrap()
                .into_iter()
                .map(|(block_header, _has_notes)| block_header)
                .collect();
        assert_eq!(
            &[mock_block_headers[1].clone(), mock_block_headers[3].clone()],
            &block_headers[..]
        );
    }

    /// Tests that large stored MMRs are built consistently throughout multiple prunes
    #[tokio::test]
    async fn partial_mmr_reconstructs_after_multiple_prune() {
        // Setup (mock a large MMR to work with, with a partial tracked set)
        // ----------------------------------------------------------------------------------------

        let store = create_test_store().await;
        const TOTAL_BLOCKS: usize = 7300;

        let tx_kernel_commitment = TransactionKernel.to_commitment();
        let block_headers: Vec<BlockHeader> = (0..TOTAL_BLOCKS)
            .map(|block_num| {
                BlockHeader::mock(
                    u32::try_from(block_num).unwrap(),
                    None,
                    None,
                    &[],
                    tx_kernel_commitment,
                )
            })
            .collect();

        let mut mmr = Mmr::default();
        for header in &block_headers {
            mmr.add(header.commitment());
        }

        let mut tracked_set: BTreeSet<usize> = (0..(TOTAL_BLOCKS - 1)).step_by(97).collect();
        tracked_set.insert(TOTAL_BLOCKS - 2);
        let tracked_blocks: Vec<usize> = tracked_set.iter().copied().collect();

        let mut tracked_nodes: BTreeMap<InOrderIndex, Word> = BTreeMap::new();
        for &block_num in &tracked_blocks {
            let header = &block_headers[block_num];
            tracked_nodes.insert(InOrderIndex::from_leaf_pos(block_num), header.commitment());

            let proof = mmr.open(block_num).expect("valid proof");
            let mut idx = InOrderIndex::from_leaf_pos(block_num);
            for node in proof.merkle_path.nodes() {
                tracked_nodes.insert(idx.sibling(), *node);
                idx = idx.parent();
            }
        }
        let tracked_nodes: Vec<(InOrderIndex, Word)> = tracked_nodes.into_iter().collect();

        let peaks_by_block: Vec<MmrPeaks> = (0..TOTAL_BLOCKS)
            .map(|block_num| mmr.peaks_at(Forest::new(block_num)).expect("valid peaks"))
            .collect();

        // Save blocks and nodes
        store
            .interact_with_connection(move |conn| {
                let tx = conn.transaction().unwrap();
                for block_num in 0..TOTAL_BLOCKS {
                    let has_notes = tracked_set.contains(&block_num);
                    SqliteStore::insert_block_header_tx(
                        &tx,
                        &block_headers[block_num],
                        &peaks_by_block[block_num],
                        has_notes,
                    )
                    .unwrap();
                }

                SqliteStore::insert_partial_blockchain_nodes_tx(&tx, &tracked_nodes).unwrap();
                tx.commit().unwrap();
                Ok(())
            })
            .await
            .unwrap();

        let prune_heights = [
            TOTAL_BLOCKS / 5,
            (TOTAL_BLOCKS * 2) / 5,
            (TOTAL_BLOCKS * 3) / 5,
            TOTAL_BLOCKS - 1,
        ];

        // Tests/assertions
        // ----------------------------------------------------------------------------------------

        let mut previous_remaining: Option<i64> = None;
        for height in prune_heights {
            let height_i64 = i64::try_from(height).expect("fits in i64");

            // Update sync height to simulate having synced further
            store
                .interact_with_connection(move |conn| {
                    conn.execute("UPDATE state_sync SET block_num = ?", params![height_i64])
                        .unwrap();
                    Ok(())
                })
                .await
                .unwrap();

            // Prune
            store.prune_irrelevant_blocks().await.unwrap();

            // Assert blocks
            let remaining_headers: i64 = store
                .interact_with_connection(|conn| {
                    let count = conn
                        .query_row("SELECT COUNT(*) FROM block_headers", [], |row| row.get(0))
                        .unwrap();
                    Ok(count)
                })
                .await
                .unwrap();
            if let Some(previous) = previous_remaining {
                assert!(remaining_headers < previous);
            } else {
                assert!(remaining_headers < i64::try_from(TOTAL_BLOCKS).unwrap());
            }
            previous_remaining = Some(remaining_headers);
        }

        // Try build MMR
        let partial_mmr = Store::get_current_partial_mmr(&store).await.unwrap();
        assert_eq!(partial_mmr.peaks().hash_peaks(), mmr.peaks().hash_peaks());

        for block_num in tracked_blocks {
            let partial_proof = partial_mmr.open(block_num).expect("partial mmr query succeeds");
            assert!(partial_proof.is_some());
            assert_eq!(partial_proof.unwrap(), mmr.open(block_num).unwrap());
        }
    }
}
