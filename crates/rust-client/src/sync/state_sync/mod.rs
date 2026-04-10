use alloc::boxed::Box;
use alloc::collections::BTreeSet;
use alloc::sync::Arc;
use alloc::vec::Vec;

use async_trait::async_trait;
use miden_protocol::account::{AccountHeader, AccountId};
use miden_protocol::crypto::merkle::mmr::PartialMmr;
use miden_protocol::note::NoteTag;

use super::state_sync_update::TransactionUpdateTracker;
use super::{BlockUpdates, StateSyncUpdate};
use crate::ClientError;
use crate::note::NoteUpdateTracker;
use crate::rpc::NodeRpcClient;
use crate::rpc::domain::note::CommittedNote;
use crate::store::{InputNoteRecord, OutputNoteRecord};
use crate::transaction::TransactionRecord;

mod apply;
mod fetch;
use fetch::SyncData;

// SYNC REQUEST
// ================================================================================================

/// Bundles the client state needed to perform a sync operation.
///
/// The sync process uses these inputs to:
/// - Request account commitment updates from the node for the provided accounts.
/// - Filter which note inclusions the node returns based on the provided note tags.
/// - Follow the lifecycle of every tracked note (input and output), transitioning them from pending
///   to committed to consumed as the network state advances.
/// - Track uncommitted transactions so they can be marked as committed when the node confirms them,
///   or discarded when they become stale.
///
/// Use [`Client::build_sync_input()`](`crate::Client::build_sync_input()`) to build a default input
/// from the client state, or construct this struct manually for custom sync scenarios.
pub struct StateSyncInput {
    /// Account headers to request commitment updates for.
    pub accounts: Vec<AccountHeader>,
    /// Note tags that the node uses to filter which note inclusions to return.
    pub note_tags: BTreeSet<NoteTag>,
    /// Input notes whose lifecycle should be followed during sync.
    pub input_notes: Vec<InputNoteRecord>,
    /// Output notes whose lifecycle should be followed during sync.
    pub output_notes: Vec<OutputNoteRecord>,
    /// Transactions to track for commitment or discard during sync.
    pub uncommitted_transactions: Vec<TransactionRecord>,
}

// SYNC CALLBACKS
// ================================================================================================

/// The action to be taken when a note update is received as part of the sync response.
#[allow(clippy::large_enum_variant)]
pub enum NoteUpdateAction {
    /// The note commit update is relevant and the specified note should be marked as committed in
    /// the store, storing its inclusion proof.
    Commit(CommittedNote),
    /// The public note is relevant and should be inserted into the store.
    Insert(InputNoteRecord),
    /// The note update is not relevant and should be discarded.
    Discard,
}

#[async_trait(?Send)]
pub trait OnNoteReceived {
    /// Callback that gets executed when a new note is received as part of the sync response.
    ///
    /// It receives:
    ///
    /// - The committed note received from the network.
    /// - An optional note record that corresponds to the state of the note in the network (only if
    ///   the note is public).
    ///
    /// It returns an enum indicating the action to be taken for the received note update. Whether
    /// the note updated should be committed, new public note inserted, or ignored.
    async fn on_note_received(
        &self,
        committed_note: CommittedNote,
        public_note: Option<InputNoteRecord>,
    ) -> Result<NoteUpdateAction, ClientError>;
}

// STATE SYNC
// ================================================================================================

/// The state sync component encompasses the client's sync logic. It is then used to request
/// updates from the node and apply them to the relevant elements. The updates are then returned and
/// can be applied to the store to persist the changes.
#[derive(Clone)]
pub struct StateSync {
    /// The RPC client used to communicate with the node.
    rpc_api: Arc<dyn NodeRpcClient>,
    /// Responsible for checking the relevance of notes and executing the
    /// [`OnNoteReceived`] callback when a new note inclusion is received.
    note_screener: Arc<dyn OnNoteReceived>,
    /// Number of blocks after which pending transactions are considered stale and discarded.
    /// If `None`, there is no limit and transactions will be kept indefinitely.
    tx_discard_delta: Option<u32>,
    /// Whether to check for nullifiers during state sync. When enabled, the component will query
    /// the nullifiers for unspent notes at each sync step. This allows to detect when tracked
    /// notes have been consumed externally and discard local transactions that depend on them.
    sync_nullifiers: bool,
}

impl StateSync {
    /// Creates a new instance of the state sync component.
    ///
    /// The nullifiers sync is enabled by default. To disable it, see
    /// [`Self::disable_nullifier_sync`].
    ///
    /// # Arguments
    ///
    /// * `rpc_api` - The RPC client used to communicate with the node.
    /// * `note_screener` - The note screener used to check the relevance of notes.
    /// * `tx_discard_delta` - Number of blocks after which pending transactions are discarded.
    pub fn new(
        rpc_api: Arc<dyn NodeRpcClient>,
        note_screener: Arc<dyn OnNoteReceived>,
        tx_discard_delta: Option<u32>,
    ) -> Self {
        Self {
            rpc_api,
            note_screener,
            tx_discard_delta,
            sync_nullifiers: true,
        }
    }

    /// Disables the nullifier sync.
    ///
    /// When disabled, the component will not query the node for new nullifiers after each sync
    /// step. This is useful for clients that don't need to track note consumption, such as
    /// faucets.
    pub fn disable_nullifier_sync(&mut self) {
        self.sync_nullifiers = false;
    }

    /// Enables the nullifier sync.
    pub fn enable_nullifier_sync(&mut self) {
        self.sync_nullifiers = true;
    }

    /// Syncs the state of the client with the chain tip of the node, returning the updates that
    /// should be applied to the store.
    ///
    /// Use [`Client::build_sync_input()`](`crate::Client::build_sync_input()`) to build the default
    /// input, or assemble it manually for custom sync. The `current_partial_mmr` is taken by
    /// mutable reference so callers can keep it in memory across syncs.
    ///
    /// During the sync process, the following steps are performed:
    /// 1. A request is sent to the node to get the state updates. This request includes tracked
    ///    account IDs and the tags of notes that might have changed or that might be of interest to
    ///    the client.
    /// 2. A response is received with the current state of the network. The response includes
    ///    information about new and committed notes, updated accounts, and committed transactions.
    /// 3. Tracked public accounts are updated and private accounts are validated against the node
    ///    state.
    /// 4. Tracked notes are updated with their new states. Notes might be committed or nullified
    ///    during the sync processing.
    /// 5. New notes are checked, and only relevant ones are stored. Relevance is determined by the
    ///    [`OnNoteReceived`] callback.
    /// 6. Transactions are updated with their new states. Transactions might be committed or
    ///    discarded.
    /// 7. The MMR is updated with the new peaks and authentication nodes.
    pub async fn sync_state(
        &self,
        current_partial_mmr: &mut PartialMmr,
        input: StateSyncInput,
    ) -> Result<StateSyncUpdate, ClientError> {
        let StateSyncInput {
            accounts,
            note_tags,
            input_notes,
            output_notes,
            uncommitted_transactions,
        } = input;
        let block_num = u32::try_from(current_partial_mmr.forest().num_leaves().saturating_sub(1))
            .map_err(|_| ClientError::InvalidPartialMmrForest)?
            .into();

        let mut update = StateSyncUpdate {
            block_num,
            note_updates: NoteUpdateTracker::new(input_notes, output_notes),
            transaction_updates: TransactionUpdateTracker::new(uncommitted_transactions),
            ..Default::default()
        };

        let note_tags = Arc::new(note_tags);
        let account_ids: Vec<AccountId> = accounts.iter().map(AccountHeader::id).collect();
        let Some(sync_data) = self
            .fetch_sync_data(update.block_num, &account_ids, &note_tags)
            .await?
        else {
            return Ok(update);
        };

        let SyncData {
            mmr_delta,
            chain_tip_header,
            note_blocks,
            public_notes,
            account_commitment_updates,
            transactions,
            nullifiers,
        } = sync_data;

        update.block_num = chain_tip_header.block_num();

        // Step 1: Advance MMR to chain tip.
        Self::advance_mmr(
            mmr_delta,
            &chain_tip_header,
            &mut update.block_updates,
            current_partial_mmr,
        )?;

        // Step 2: Screen note inclusions and track relevant blocks in MMR.
        self.process_note_inclusions(
            note_blocks,
            public_notes,
            &mut update,
            current_partial_mmr,
        )
        .await?;

        // Step 3: Update account states (fetches updated public accounts from node).
        self.process_account_updates(
            &mut update.account_updates,
            &accounts,
            &account_commitment_updates,
        )
        .await?;

        // Step 4: Process transaction inclusions, nullifier ordering, and output note commits.
        update.apply_transaction_data(
            &chain_tip_header,
            &transactions,
            nullifiers,
            self.tx_discard_delta,
        )?;

        // Step 5: Detect notes consumed externally via nullifier sync.
        if self.sync_nullifiers {
            self.sync_consumed_notes(&mut update, block_num).await?;
        }

        Ok(update)
    }
}

#[cfg(test)]
mod tests {
    use alloc::collections::BTreeSet;
    use alloc::sync::Arc;

    use async_trait::async_trait;
    use miden_protocol::assembly::DefaultSourceManager;
    use miden_protocol::crypto::merkle::mmr::{Forest, PartialMmr};
    use miden_protocol::note::{NoteTag, NoteType};
    use miden_protocol::{Felt, Word};
    use miden_standards::code_builder::CodeBuilder;
    use miden_testing::MockChainBuilder;

    use super::*;
    use crate::rpc::domain::note::CommittedNote;
    use crate::store::StoreError;
    use crate::testing::mock::MockRpcApi;

    /// Mock note screener that discards all notes, for minimal test setup.
    struct MockScreener;

    #[async_trait(?Send)]
    impl OnNoteReceived for MockScreener {
        async fn on_note_received(
            &self,
            _committed_note: CommittedNote,
            _public_note: Option<InputNoteRecord>,
        ) -> Result<NoteUpdateAction, ClientError> {
            Ok(NoteUpdateAction::Discard)
        }
    }

    fn empty() -> StateSyncInput {
        StateSyncInput {
            accounts: vec![],
            note_tags: BTreeSet::new(),
            input_notes: vec![],
            output_notes: vec![],
            uncommitted_transactions: vec![],
        }
    }

    // CONSUMED NOTE ORDERING INTEGRATION TESTS
    // --------------------------------------------------------------------------------------------

    /// Mock note screener that commits all notes matching tracked input notes.
    struct CommitAllScreener;

    #[async_trait(?Send)]
    impl OnNoteReceived for CommitAllScreener {
        async fn on_note_received(
            &self,
            committed_note: CommittedNote,
            _public_note: Option<InputNoteRecord>,
        ) -> Result<NoteUpdateAction, ClientError> {
            Ok(NoteUpdateAction::Commit(committed_note))
        }
    }

    use miden_protocol::account::{Account, AccountId};
    use miden_protocol::note::{Note, NoteId};

    /// Builds a `MockChain` where 3 notes are consumed by chained transactions in the same block.
    async fn build_chain_with_chained_consume_txs() -> (miden_testing::MockChain, Account, [Note; 3])
    {
        use miden_protocol::asset::{Asset, FungibleAsset};
        use miden_protocol::note::NoteType;
        use miden_protocol::testing::account_id::{
            ACCOUNT_ID_PRIVATE_FUNGIBLE_FAUCET,
            ACCOUNT_ID_SENDER,
        };
        use miden_testing::{MockChainBuilder, TxContextInput};

        let sender_id: AccountId = ACCOUNT_ID_SENDER.try_into().unwrap();
        let faucet_id: AccountId = ACCOUNT_ID_PRIVATE_FUNGIBLE_FAUCET.try_into().unwrap();

        let mut builder = MockChainBuilder::new();
        let account = builder.add_existing_mock_account(miden_testing::Auth::IncrNonce).unwrap();
        let account_id = account.id();

        let asset = Asset::Fungible(FungibleAsset::new(faucet_id, 100u64).unwrap());
        let note1 = builder
            .add_p2id_note(sender_id, account_id, &[asset], NoteType::Public)
            .unwrap();
        let note2 = builder
            .add_p2id_note(sender_id, account_id, &[asset], NoteType::Public)
            .unwrap();
        let note3 = builder
            .add_p2id_note(sender_id, account_id, &[asset], NoteType::Public)
            .unwrap();

        let mut chain = builder.build().unwrap();
        chain.prove_next_block().unwrap();

        let mut current_account = account.clone();
        for note in [&note1, &note2, &note3] {
            let tx = Box::pin(
                chain
                    .build_tx_context(
                        TxContextInput::Account(current_account.clone()),
                        &[],
                        core::slice::from_ref(note),
                    )
                    .unwrap()
                    .build()
                    .unwrap()
                    .execute(),
            )
            .await
            .unwrap();
            current_account.apply_delta(tx.account_delta()).unwrap();
            chain.add_pending_executed_transaction(&tx).unwrap();
        }

        chain.prove_next_block().unwrap();
        (chain, account, [note1, note2, note3])
    }

    #[tokio::test]
    async fn sync_state_sets_consumed_tx_order_for_chained_transactions() {
        use miden_protocol::note::NoteMetadata;

        let (chain, account, [note1, note2, note3]) = build_chain_with_chained_consume_txs().await;

        let mock_rpc = MockRpcApi::new(chain);
        let state_sync =
            StateSync::new(Arc::new(mock_rpc.clone()), Arc::new(CommitAllScreener), None);

        let genesis_peaks = mock_rpc.get_mmr().peaks_at(Forest::new(1)).unwrap();
        let mut partial_mmr = PartialMmr::from_peaks(genesis_peaks);

        let input_notes: Vec<InputNoteRecord> = [&note1, &note2, &note3]
            .into_iter()
            .map(|n| InputNoteRecord::from(n.clone()))
            .collect();

        let note_tags: BTreeSet<NoteTag> =
            input_notes.iter().filter_map(|n| n.metadata().map(NoteMetadata::tag)).collect();

        let account_id = account.id();
        let sync_input = StateSyncInput {
            accounts: vec![account.into()],
            note_tags,
            input_notes,
            output_notes: vec![],
            uncommitted_transactions: vec![],
        };

        let update = state_sync.sync_state(&mut partial_mmr, sync_input).await.unwrap();

        let updated_notes: Vec<_> = update.note_updates.updated_input_notes().collect();

        let find_order = |note_id: NoteId| -> Option<u32> {
            updated_notes
                .iter()
                .find(|n| n.id() == note_id)
                .and_then(|n| n.consumed_tx_order())
        };

        assert_eq!(find_order(note1.id()), Some(0), "note1 should have tx_order 0");
        assert_eq!(find_order(note2.id()), Some(1), "note2 should have tx_order 1");
        assert_eq!(find_order(note3.id()), Some(2), "note3 should have tx_order 2");

        for note in &updated_notes {
            let record = note.inner();
            assert!(record.is_consumed(), "note should be in a consumed state");
            assert_eq!(
                record.consumer_account(),
                Some(account_id),
                "externally-consumed notes by a tracked account should have consumer_account set",
            );
        }
    }

    #[tokio::test]
    async fn sync_state_across_multiple_iterations_with_same_mmr() {
        let mock_rpc = MockRpcApi::default();
        mock_rpc.advance_blocks(3);
        let chain_tip_1 = mock_rpc.get_chain_tip_block_num();

        let state_sync = StateSync::new(Arc::new(mock_rpc.clone()), Arc::new(MockScreener), None);

        let genesis_peaks = mock_rpc.get_mmr().peaks_at(Forest::new(1)).unwrap();
        let mut partial_mmr = PartialMmr::from_peaks(genesis_peaks);
        assert_eq!(partial_mmr.forest().num_leaves(), 1);

        let update = state_sync.sync_state(&mut partial_mmr, empty()).await.unwrap();

        assert_eq!(update.block_num, chain_tip_1);
        let forest_1 = partial_mmr.forest();
        assert_eq!(forest_1.num_leaves(), chain_tip_1.as_u32() as usize + 1);

        mock_rpc.advance_blocks(2);
        let chain_tip_2 = mock_rpc.get_chain_tip_block_num();

        let update = state_sync.sync_state(&mut partial_mmr, empty()).await.unwrap();

        assert_eq!(update.block_num, chain_tip_2);
        let forest_2 = partial_mmr.forest();
        assert!(forest_2 > forest_1);
        assert_eq!(forest_2.num_leaves(), chain_tip_2.as_u32() as usize + 1);

        let update = state_sync.sync_state(&mut partial_mmr, empty()).await.unwrap();

        assert_eq!(update.block_num, chain_tip_2);
        assert_eq!(partial_mmr.forest(), forest_2);
    }

    async fn build_chain_with_mint_notes(
        num_blocks: u64,
    ) -> (miden_testing::MockChain, BTreeSet<NoteTag>) {
        let mut builder = MockChainBuilder::new();
        let faucet = builder
            .add_existing_basic_faucet(
                miden_testing::Auth::BasicAuth {
                    auth_scheme: miden_protocol::account::auth::AuthScheme::Falcon512Poseidon2,
                },
                "TST",
                10_000,
                None,
            )
            .unwrap();
        let _target = builder.add_existing_mock_account(miden_testing::Auth::IncrNonce).unwrap();
        let mut chain = builder.build().unwrap();

        let recipient: Word = [0u32, 1, 2, 3].into();
        let tag = NoteTag::default();
        let mut faucet_account = faucet.clone();
        let mut note_tags = BTreeSet::new();

        for i in 0..num_blocks {
            let amount = Felt::new(100 + i);
            let source_manager = Arc::new(DefaultSourceManager::default());
            let tx_script_code = format!(
                "
                begin
                    padw padw push.0
                    push.{r0}.{r1}.{r2}.{r3}
                    push.{note_type}
                    push.{tag}
                    push.{amount}
                    call.::miden::standards::faucets::basic_fungible::mint_and_send
                    dropw dropw dropw dropw
                end
                ",
                r0 = recipient[0],
                r1 = recipient[1],
                r2 = recipient[2],
                r3 = recipient[3],
                note_type = NoteType::Private as u8,
                tag = u32::from(tag),
                amount = amount,
            );
            let tx_script = CodeBuilder::with_source_manager(source_manager.clone())
                .compile_tx_script(tx_script_code)
                .unwrap();
            let tx = Box::pin(
                chain
                    .build_tx_context(
                        miden_testing::TxContextInput::Account(faucet_account.clone()),
                        &[],
                        &[],
                    )
                    .unwrap()
                    .tx_script(tx_script)
                    .with_source_manager(source_manager)
                    .build()
                    .unwrap()
                    .execute(),
            )
            .await
            .unwrap();

            for output_note in tx.output_notes().iter() {
                note_tags.insert(output_note.metadata().tag());
            }

            faucet_account.apply_delta(tx.account_delta()).unwrap();
            chain.add_pending_executed_transaction(&tx).unwrap();
            chain.prove_next_block().unwrap();
        }

        (chain, note_tags)
    }

    #[tokio::test]
    async fn sync_state_tracks_note_blocks_in_mmr() {
        use miden_protocol::block::BlockNumber;
        use miden_protocol::crypto::merkle::mmr::InOrderIndex;

        let (chain, note_tags) = build_chain_with_mint_notes(3).await;
        let mock_rpc = MockRpcApi::new(chain);
        let chain_tip = mock_rpc.get_chain_tip_block_num();

        let note_sync =
            mock_rpc.sync_notes(BlockNumber::from(0u32), None, &note_tags).await.unwrap();
        assert!(
            note_sync.blocks.len() >= 2,
            "expected notes in multiple blocks, got {}",
            note_sync.blocks.len()
        );

        let note_block_nums: BTreeSet<BlockNumber> =
            note_sync.blocks.iter().map(|b| b.block_header.block_num()).collect();

        let state_sync = StateSync::new(Arc::new(mock_rpc.clone()), Arc::new(MockScreener), None);

        let genesis_peaks = mock_rpc.get_mmr().peaks_at(Forest::new(1)).unwrap();
        let mut partial_mmr = PartialMmr::from_peaks(genesis_peaks);

        let sync_data = state_sync
            .fetch_sync_data(BlockNumber::GENESIS, &[], &Arc::new(note_tags.clone()))
            .await
            .unwrap()
            .expect("should have progressed past genesis");

        assert_eq!(sync_data.chain_tip_header.block_num(), chain_tip);
        assert!(!sync_data.note_blocks.is_empty(), "should have note blocks");

        let _auth_nodes: Vec<(InOrderIndex, Word)> =
            partial_mmr.apply(sync_data.mmr_delta).map_err(StoreError::MmrError).unwrap();
        partial_mmr.add(sync_data.chain_tip_header.commitment(), false);

        assert_eq!(partial_mmr.forest().num_leaves(), chain_tip.as_u32() as usize + 1);

        for block in &sync_data.note_blocks {
            let bn = block.block_header.block_num();
            partial_mmr
                .track(bn.as_usize(), block.block_header.commitment(), &block.mmr_path)
                .map_err(StoreError::MmrError)
                .unwrap();

            assert!(
                partial_mmr.is_tracked(bn.as_usize()),
                "block {bn} should be tracked after calling track()"
            );
        }

        for &bn in &note_block_nums {
            assert!(
                partial_mmr.is_tracked(bn.as_usize()),
                "block {bn} with notes should be tracked in partial MMR"
            );
        }
    }
}
