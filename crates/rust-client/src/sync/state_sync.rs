use alloc::boxed::Box;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::sync::Arc;
use alloc::vec::Vec;

use async_trait::async_trait;
use miden_protocol::Word;
use miden_protocol::account::{AccountHeader, AccountId};
use miden_protocol::block::{BlockHeader, BlockNumber};
use miden_protocol::crypto::merkle::mmr::{InOrderIndex, MmrDelta, MmrPeaks, PartialMmr};
use miden_protocol::note::{NoteId, NoteTag, Nullifier};
use tracing::info;

use super::state_sync_update::TransactionUpdateTracker;
use super::{AccountUpdates, StateSyncUpdate};
use crate::ClientError;
use crate::note::NoteUpdateTracker;
use crate::rpc::NodeRpcClient;
use crate::rpc::domain::account::FetchedAccount;
use crate::rpc::domain::note::CommittedNote;
use crate::rpc::domain::sync::StateSyncInfo;
use crate::rpc::domain::transaction::{
    TransactionInclusion,
    TransactionRecord as RpcTransactionRecord,
};
use crate::store::{InputNoteRecord, OutputNoteRecord, StoreError};
use crate::transaction::TransactionRecord;

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
    #[allow(clippy::too_many_lines)]
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
        let block_num = u32::try_from(
            current_partial_mmr.forest().num_leaves().checked_sub(1).unwrap_or_default(),
        )
        .map_err(|_| ClientError::InvalidPartialMmrForest)?
        .into();

        let mut state_sync_update = StateSyncUpdate {
            block_num,
            note_updates: NoteUpdateTracker::new(input_notes, output_notes),
            transaction_updates: TransactionUpdateTracker::new(uncommitted_transactions),
            ..Default::default()
        };

        let note_tags = Arc::new(note_tags);
        let account_ids: Vec<AccountId> = accounts.iter().map(AccountHeader::id).collect();
        let mut state_sync_steps = Vec::new();

        while let Some(step) = self
            .sync_state_step(state_sync_update.block_num, &account_ids, &note_tags)
            .await?
        {
            let sync_block_num = step.block_header.block_num();

            let reached_tip = step.chain_tip == sync_block_num;

            state_sync_update.block_num = sync_block_num;
            state_sync_steps.push(step);

            if reached_tip {
                break;
            }
        }

        // TODO: fetch_public_note_details should take an iterator or btreeset down to the RPC call
        // (this would be a breaking change so it should be done separately)
        let public_note_ids: Vec<NoteId> = state_sync_steps
            .iter()
            .flat_map(|s| s.note_inclusions.iter())
            .filter(|n| !n.metadata().is_private())
            .map(|n| *n.note_id())
            .collect();

        let public_note_records = self.fetch_public_note_details(&public_note_ids).await?;

        // Collect account commitment updates across all sync steps. Each account only needs
        // to be checked once since GetAccount always returns the latest state.
        let merged_commitment_updates: Vec<(AccountId, Word)> = state_sync_steps
            .iter()
            .flat_map(|s| s.account_commitment_updates.iter())
            .map(|(id, w)| (*id, *w))
            .collect::<BTreeMap<_, _>>()
            .into_iter()
            .collect();

        self.account_state_sync(
            &mut state_sync_update.account_updates,
            &accounts,
            &merged_commitment_updates,
        )
        .await?;

        // Apply local changes. These involve updating the MMR and applying state transitions
        // to notes based on the received information.
        info!("Applying state transitions locally.");

        self.apply_sync_steps(
            state_sync_steps,
            &public_note_records,
            &mut state_sync_update,
            current_partial_mmr,
        )
        .await?;

        if self.sync_nullifiers {
            info!("Syncing nullifiers.");
            self.nullifiers_state_sync(&mut state_sync_update, block_num).await?;
        }

        Ok(state_sync_update)
    }

    /// Executes a single sync step by composing calls to `sync_notes`, `sync_chain_mmr`, and
    /// `sync_transactions`.
    ///
    /// `sync_notes` drives the loop: it determines the target block (the first block containing
    /// a matching note, or the chain tip). If the target block equals `current_block_num`, `None`
    /// is returned, signalling that the client is already at the requested height.
    ///
    /// The other two calls use the same target block to ensure a consistent range.
    async fn sync_state_step(
        &self,
        current_block_num: BlockNumber,
        account_ids: &[AccountId],
        note_tags: &Arc<BTreeSet<NoteTag>>,
    ) -> Result<Option<StateSyncInfo>, ClientError> {
        info!("Performing sync state step.");

        // Retrieve sync_notes
        let note_sync =
            self.rpc_api.sync_notes(current_block_num, None, note_tags.as_ref()).await?;

        let target_block = note_sync.block_header.block_num();

        // We don't need to continue if the chain has not advanced
        if target_block == current_block_num {
            return Ok(None);
        }

        // Get MMR delta for the same range
        let mmr_delta = self
            .rpc_api
            .sync_chain_mmr(current_block_num, Some(target_block))
            .await?
            .mmr_delta;

        // Gather transactions for tracked accounts (skip if none)
        let (account_commitment_updates, transactions, nullifiers) = if account_ids.is_empty() {
            (vec![], vec![], vec![])
        } else {
            let tx_info = self
                .rpc_api
                .sync_transactions(current_block_num, Some(target_block), account_ids.to_vec())
                .await?;

            let account_updates = derive_account_commitment_updates(&tx_info.transaction_records);

            let tx_inclusions = tx_info
                .transaction_records
                .iter()
                .map(|r| TransactionInclusion {
                    transaction_id: r.transaction_header.id(),
                    block_num: r.block_num,
                    account_id: r.transaction_header.account_id(),
                    initial_state_commitment: r.transaction_header.initial_state_commitment(),
                })
                .collect();

            let nullifiers = compute_ordered_nullifiers(&tx_info.transaction_records);

            (account_updates, tx_inclusions, nullifiers)
        };

        // Compose StateSyncInfo with sync results
        Ok(Some(StateSyncInfo {
            chain_tip: note_sync.chain_tip,
            block_header: note_sync.block_header,
            mmr_delta,
            account_commitment_updates,
            note_inclusions: note_sync.notes,
            transactions,
            nullifiers,
        }))
    }

    // HELPERS
    // --------------------------------------------------------------------------------------------

    /// Applies each sync step to the update.
    async fn apply_sync_steps(
        &self,
        sync_steps: Vec<StateSyncInfo>,
        public_note_records: &BTreeMap<NoteId, InputNoteRecord>,
        state_sync_update: &mut StateSyncUpdate,
        current_partial_mmr: &mut PartialMmr,
    ) -> Result<(), ClientError> {
        for sync_step in sync_steps {
            let StateSyncInfo {
                chain_tip,
                block_header,
                mmr_delta,
                note_inclusions,
                transactions,
                nullifiers,
                ..
            } = sync_step;

            state_sync_update.note_updates.extend_nullifiers(nullifiers);

            self.transaction_state_sync(
                &mut state_sync_update.transaction_updates,
                &block_header,
                &transactions,
            );

            let found_relevant_note = self
                .note_state_sync(
                    &mut state_sync_update.note_updates,
                    note_inclusions,
                    &block_header,
                    public_note_records,
                )
                .await?;

            let (new_mmr_peaks, new_authentication_nodes) = apply_mmr_changes(
                &block_header,
                found_relevant_note,
                current_partial_mmr,
                mmr_delta,
            )?;

            let include_block = found_relevant_note || chain_tip == block_header.block_num();
            if include_block {
                state_sync_update.block_updates.insert(
                    block_header,
                    found_relevant_note,
                    new_mmr_peaks,
                    new_authentication_nodes,
                );
            } else {
                // Even though this block header is not stored, `apply_mmr_changes` may
                // produce authentication nodes for already-tracked leaves whose Merkle
                // paths change as the MMR grows. These must be persisted so that the
                // `PartialMmr` can be correctly reconstructed from the store after a
                // client restart.
                state_sync_update
                    .block_updates
                    .extend_authentication_nodes(new_authentication_nodes);
            }
        }

        Ok(())
    }

    /// Compares the state of tracked accounts with the updates received from the node. The method
    /// Updates the `account_updates` with the details of the accounts that need to be updated.
    ///
    /// The account updates might include:
    /// * Public accounts that have been updated in the node (full or delta-based).
    /// * Network accounts that have been updated in the node and are being tracked by the client.
    /// * Private accounts that have been marked as mismatched because the current commitment
    ///   doesn't match the one received from the node. The client will need to handle these cases
    ///   as they could be a stale account state or a reason to lock the account.
    async fn account_state_sync(
        &self,
        account_updates: &mut AccountUpdates,
        accounts: &[AccountHeader],
        account_commitment_updates: &[(AccountId, Word)],
    ) -> Result<(), ClientError> {
        let (public_accounts, private_accounts): (Vec<_>, Vec<_>) =
            accounts.iter().partition(|account_header| !account_header.id().is_private());

        self.sync_public_accounts(account_updates, account_commitment_updates, &public_accounts)
            .await?;

        let mismatched_private_accounts = account_commitment_updates
            .iter()
            .filter(|(account_id, digest)| {
                private_accounts.iter().any(|account| {
                    account.id() == *account_id && &account.to_commitment() != digest
                })
            })
            .copied()
            .collect::<Vec<_>>();

        account_updates.extend(AccountUpdates::new(Vec::new(), mismatched_private_accounts));

        Ok(())
    }

    /// Queries the node for updated public accounts and populates `account_updates`.
    ///
    /// For each mismatched public account, calls `get_account_details` (which internally
    /// handles oversized maps and vaults) and adds the full account to the update list.
    async fn sync_public_accounts(
        &self,
        account_updates: &mut AccountUpdates,
        commitment_updates: &[(AccountId, Word)],
        current_public_accounts: &[&AccountHeader],
    ) -> Result<(), ClientError> {
        for (id, commitment) in commitment_updates {
            let Some(local_account) = current_public_accounts
                .iter()
                .find(|acc| *id == acc.id() && *commitment != acc.to_commitment())
            else {
                continue;
            };

            let response = self
                .rpc_api
                .get_account_details(local_account.id())
                .await
                .map_err(ClientError::RpcError)?;

            match response {
                FetchedAccount::Public(account, _) => {
                    let account = *account;
                    // Only update if the account is newer.
                    if account.nonce().as_int() > local_account.nonce().as_int() {
                        account_updates.extend(AccountUpdates::new(vec![account], Vec::new()));
                    }
                },
                FetchedAccount::Private(..) => {
                    // Should not happen for public accounts, skip silently.
                },
            }
        }

        Ok(())
    }

    /// Applies the changes received from the sync response to the notes and transactions tracked
    /// by the client and updates the `note_updates` accordingly.
    ///
    /// This method uses the callbacks provided to the [`StateSync`] component to check if the
    /// updates received are relevant to the client.
    ///
    /// The note updates might include:
    /// * New notes that we received from the node and might be relevant to the client.
    /// * Tracked expected notes that were committed in the block.
    /// * Tracked notes that were being processed by a transaction that got committed.
    /// * Tracked notes that were nullified by an external transaction.
    ///
    /// The `public_notes` parameter provides cached public note details for the current sync
    /// iteration so the node is only queried once per batch.
    async fn note_state_sync(
        &self,
        note_updates: &mut NoteUpdateTracker,
        note_inclusions: Vec<CommittedNote>,
        block_header: &BlockHeader,
        public_notes: &BTreeMap<NoteId, InputNoteRecord>,
    ) -> Result<bool, ClientError> {
        // `found_relevant_note` tracks whether we want to persist the block header in the end
        let mut found_relevant_note = false;

        for committed_note in note_inclusions {
            let public_note = (!committed_note.metadata().is_private())
                .then(|| public_notes.get(committed_note.note_id()))
                .flatten()
                .cloned();

            match self.note_screener.on_note_received(committed_note, public_note).await? {
                NoteUpdateAction::Commit(committed_note) => {
                    // Only mark the downloaded block header as relevant if we are talking about
                    // an input note (output notes get marked as committed but we don't need the
                    // block for anything there)
                    found_relevant_note |= note_updates
                        .apply_committed_note_state_transitions(&committed_note, block_header)?;
                },
                NoteUpdateAction::Insert(public_note) => {
                    found_relevant_note = true;

                    note_updates.apply_new_public_note(public_note, block_header)?;
                },
                NoteUpdateAction::Discard => {},
            }
        }

        Ok(found_relevant_note)
    }

    /// Queries the node for all received notes that aren't being locally tracked in the client.
    ///
    /// The client can receive metadata for private notes that it's not tracking. In this case,
    /// notes are ignored for now as they become useless until details are imported.
    async fn fetch_public_note_details(
        &self,
        query_notes: &[NoteId],
    ) -> Result<BTreeMap<NoteId, InputNoteRecord>, ClientError> {
        if query_notes.is_empty() {
            return Ok(BTreeMap::new());
        }

        info!("Getting note details for notes that are not being tracked.");

        let return_notes = self.rpc_api.get_public_note_records(query_notes, None).await?;

        Ok(return_notes.into_iter().map(|note| (note.id(), note)).collect())
    }

    /// Collects the nullifier tags for the notes that were updated in the sync response and uses
    /// the `sync_nullifiers` endpoint to check if there are new nullifiers for these
    /// notes. It then processes the nullifiers to apply the state transitions on the note updates.
    ///
    /// The `state_sync_update` parameter will be updated to track the new discarded transactions.
    async fn nullifiers_state_sync(
        &self,
        state_sync_update: &mut StateSyncUpdate,
        current_block_num: BlockNumber,
    ) -> Result<(), ClientError> {
        // To receive information about added nullifiers, we reduce them to the higher 16 bits
        // Note that besides filtering by nullifier prefixes, the node also filters by block number
        // (it only returns nullifiers from current_block_num until
        // response.block_header.block_num())

        // Check for new nullifiers for input notes that were updated
        let nullifiers_tags: Vec<u16> = state_sync_update
            .note_updates
            .unspent_nullifiers()
            .map(|nullifier| nullifier.prefix())
            .collect();

        let mut new_nullifiers = self
            .rpc_api
            .sync_nullifiers(&nullifiers_tags, current_block_num, Some(state_sync_update.block_num))
            .await?;

        // Discard nullifiers that are newer than the current block (this might happen if the block
        // changes between the sync_state and the check_nullifier calls)
        new_nullifiers.retain(|update| update.block_num <= state_sync_update.block_num);

        for nullifier_update in new_nullifiers {
            state_sync_update.note_updates.apply_nullifiers_state_transitions(
                &nullifier_update,
                state_sync_update.transaction_updates.committed_transactions(),
            )?;

            // Process nullifiers and track the updates of local tracked transactions that were
            // discarded because the notes that they were processing were nullified by an
            // another transaction.
            state_sync_update
                .transaction_updates
                .apply_input_note_nullified(nullifier_update.nullifier);
        }

        Ok(())
    }

    /// Applies the changes received from the sync response to the transactions tracked by the
    /// client and updates the `transaction_updates` accordingly.
    ///
    /// The transaction updates might include:
    /// * New transactions that were committed in the block.
    /// * Transactions that were discarded because they were stale or expired.
    fn transaction_state_sync(
        &self,
        transaction_updates: &mut TransactionUpdateTracker,
        new_block_header: &BlockHeader,
        transaction_inclusions: &[TransactionInclusion],
    ) {
        for transaction_inclusion in transaction_inclusions {
            transaction_updates.apply_transaction_inclusion(
                transaction_inclusion,
                u64::from(new_block_header.timestamp()),
            ); //TODO: Change timestamps from u64 to u32
        }

        transaction_updates
            .apply_sync_height_update(new_block_header.block_num(), self.tx_discard_delta);
    }
}

// HELPERS
// ================================================================================================

/// Derives account commitment updates from transaction records.
///
/// For each unique account, takes the `final_state_commitment` from the transaction with the
/// highest `block_num`. This replicates the old `SyncState` behavior where the node returned
/// the latest account commitment per account in the synced range.
fn derive_account_commitment_updates(
    transaction_records: &[RpcTransactionRecord],
) -> Vec<(AccountId, Word)> {
    let mut latest_by_account: BTreeMap<AccountId, &RpcTransactionRecord> = BTreeMap::new();

    for record in transaction_records {
        let account_id = record.transaction_header.account_id();
        latest_by_account
            .entry(account_id)
            .and_modify(|existing| {
                if record.block_num > existing.block_num {
                    *existing = record;
                }
            })
            .or_insert(record);
    }

    latest_by_account
        .into_iter()
        .map(|(account_id, record)| {
            (account_id, record.transaction_header.final_state_commitment())
        })
        .collect()
}

/// Applies changes to the current MMR structure, returns the updated [`MmrPeaks`] and the
/// authentication nodes for leaves we track.
fn apply_mmr_changes(
    new_block: &BlockHeader,
    new_block_has_relevant_notes: bool,
    current_partial_mmr: &mut PartialMmr,
    mmr_delta: MmrDelta,
) -> Result<(MmrPeaks, Vec<(InOrderIndex, Word)>), ClientError> {
    // Apply the MMR delta to bring MMR to forest equal to chain tip
    let mut new_authentication_nodes: Vec<(InOrderIndex, Word)> =
        current_partial_mmr.apply(mmr_delta).map_err(StoreError::MmrError)?;

    let new_peaks = current_partial_mmr.peaks();

    new_authentication_nodes
        .append(&mut current_partial_mmr.add(new_block.commitment(), new_block_has_relevant_notes));

    Ok((new_peaks, new_authentication_nodes))
}

/// Returns nullifiers ordered by consuming transaction position, per account.
///
/// Groups RPC transaction records by (`account_id`, `block_num`), chains them using
/// `initial_state_commitment` / `final_state_commitment`, and collects each transaction's
/// input note nullifiers in execution order. Nullifiers from the same account are in execution
/// order; ordering across different accounts is arbitrary.
fn compute_ordered_nullifiers(transaction_records: &[RpcTransactionRecord]) -> Vec<Nullifier> {
    // Group transactions by (account_id, block_num).
    let mut groups: BTreeMap<(AccountId, BlockNumber), Vec<&RpcTransactionRecord>> =
        BTreeMap::new();

    for record in transaction_records {
        let account_id = record.transaction_header.account_id();
        groups.entry((account_id, record.block_num)).or_default().push(record);
    }

    let mut result = Vec::new();

    for txs in groups.values() {
        // Build a lookup from initial_state_commitment -> transaction record.
        let mut init_to_tx: BTreeMap<Word, &RpcTransactionRecord> = txs
            .iter()
            .map(|tx| (tx.transaction_header.initial_state_commitment(), *tx))
            .collect();

        // Build a set of all final states to find the chain start.
        let final_states: BTreeSet<Word> =
            txs.iter().map(|tx| tx.transaction_header.final_state_commitment()).collect();

        // Find the chain start: the tx whose initial_state_commitment is not any other tx's
        // final_state_commitment.
        let chain_start = txs
            .iter()
            .find(|tx| !final_states.contains(&tx.transaction_header.initial_state_commitment()));

        let Some(start_tx) = chain_start else {
            continue;
        };

        // Walk the chain from start, removing each step from the map.
        let mut current =
            init_to_tx.remove(&start_tx.transaction_header.initial_state_commitment());

        while let Some(tx) = current {
            for commitment in tx.transaction_header.input_notes().iter() {
                result.push(commitment.nullifier());
            }
            current = init_to_tx.remove(&tx.transaction_header.final_state_commitment());
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use alloc::collections::BTreeSet;
    use alloc::sync::Arc;

    use async_trait::async_trait;
    use miden_protocol::crypto::merkle::mmr::{Forest, PartialMmr};

    use super::*;
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

    // COMPUTE NULLIFIER TX ORDER TESTS
    // --------------------------------------------------------------------------------------------

    mod compute_nullifiers_tests {
        use alloc::vec;

        use miden_protocol::asset::FungibleAsset;
        use miden_protocol::block::BlockNumber;
        use miden_protocol::note::Nullifier;
        use miden_protocol::transaction::{InputNoteCommitment, InputNotes, TransactionHeader};
        use miden_protocol::{Felt, ZERO};

        use crate::rpc::domain::transaction::{
            ACCOUNT_ID_NATIVE_ASSET_FAUCET,
            TransactionRecord as RpcTransactionRecord,
        };

        fn word(n: u64) -> miden_protocol::Word {
            [Felt::new(n), ZERO, ZERO, ZERO].into()
        }

        fn make_rpc_tx(
            init_state: u64,
            final_state: u64,
            nullifier_vals: &[u64],
            block_number: u32,
        ) -> RpcTransactionRecord {
            let account_id = miden_protocol::account::AccountId::try_from(
                miden_protocol::testing::account_id::ACCOUNT_ID_REGULAR_PRIVATE_ACCOUNT_UPDATABLE_CODE,
            )
            .unwrap();

            let input_notes = InputNotes::new_unchecked(
                nullifier_vals
                    .iter()
                    .map(|v| InputNoteCommitment::from(Nullifier::from_raw(word(*v))))
                    .collect(),
            );

            let fee =
                FungibleAsset::new(ACCOUNT_ID_NATIVE_ASSET_FAUCET.try_into().expect("valid"), 0u64)
                    .unwrap();

            RpcTransactionRecord {
                block_num: BlockNumber::from(block_number),
                transaction_header: TransactionHeader::new(
                    account_id,
                    word(init_state),
                    word(final_state),
                    input_notes,
                    vec![],
                    fee,
                ),
            }
        }

        #[test]
        fn chains_rpc_transactions_by_state_commitment() {
            // Chain: tx_a (state 1->2) -> tx_b (state 2->3) -> tx_c (state 3->4)
            // Passed in reverse order to verify chaining uses state, not insertion order.
            let tx_a = make_rpc_tx(1, 2, &[10], 5);
            let tx_b = make_rpc_tx(2, 3, &[20], 5);
            let tx_c = make_rpc_tx(3, 4, &[30], 5);

            let result = super::super::compute_ordered_nullifiers(&[tx_c, tx_a, tx_b]);

            assert_eq!(result[0], Nullifier::from_raw(word(10)));
            assert_eq!(result[1], Nullifier::from_raw(word(20)));
            assert_eq!(result[2], Nullifier::from_raw(word(30)));
        }

        #[test]
        fn groups_independently_by_account_and_block() {
            // Account A, block 5: two chained txs.
            let tx_a1 = make_rpc_tx(1, 2, &[10], 5);
            let tx_a2 = make_rpc_tx(2, 3, &[20], 5);

            // Account A, block 6: independent chain.
            let tx_a3 = make_rpc_tx(3, 4, &[30], 6);

            // Account B, block 5: independent chain.
            let account_b = miden_protocol::account::AccountId::try_from(
                miden_protocol::testing::account_id::ACCOUNT_ID_PUBLIC_FUNGIBLE_FAUCET,
            )
            .unwrap();

            let fee =
                FungibleAsset::new(ACCOUNT_ID_NATIVE_ASSET_FAUCET.try_into().expect("valid"), 0u64)
                    .unwrap();

            let tx_b1 = RpcTransactionRecord {
                block_num: BlockNumber::from(5u32),
                transaction_header: TransactionHeader::new(
                    account_b,
                    word(100),
                    word(200),
                    InputNotes::new_unchecked(vec![InputNoteCommitment::from(
                        Nullifier::from_raw(word(40)),
                    )]),
                    vec![],
                    fee,
                ),
            };

            let result = super::super::compute_ordered_nullifiers(&[tx_a2, tx_b1, tx_a3, tx_a1]);

            // Nullifiers are ordered by chain position within each (account, block) group.
            // The exact global indices depend on BTreeMap iteration order of the groups.
            let pos = |val: u64| -> usize {
                result.iter().position(|n| *n == Nullifier::from_raw(word(val))).unwrap()
            };

            // Within the same group, chain order is preserved.
            assert!(pos(10) < pos(20)); // A, block 5: pos 0 < pos 1
            // Nullifiers from different groups are all present.
            assert!(result.contains(&Nullifier::from_raw(word(30)))); // A, block 6
            assert!(result.contains(&Nullifier::from_raw(word(40)))); // B, block 5
        }

        #[test]
        fn multiple_nullifiers_per_transaction_are_consecutive() {
            // Single tx consuming 3 notes — all should appear consecutively.
            let tx = make_rpc_tx(1, 2, &[10, 20, 30], 5);

            let result = super::super::compute_ordered_nullifiers(&[tx]);

            assert_eq!(result.len(), 3);
            assert!(result.contains(&Nullifier::from_raw(word(10))));
            assert!(result.contains(&Nullifier::from_raw(word(20))));
            assert!(result.contains(&Nullifier::from_raw(word(30))));
        }

        #[test]
        fn empty_input_returns_empty_vec() {
            let result = super::super::compute_ordered_nullifiers(&[]);
            assert!(result.is_empty());
        }
    }

    // CONSUMED NOTE ORDERING INTEGRATION TESTS
    // --------------------------------------------------------------------------------------------

    /// Mock note screener that commits all notes matching tracked input notes.
    /// This ensures committed notes get their inclusion proofs set during sync.
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

    use miden_protocol::account::Account;
    use miden_protocol::note::Note;

    /// Builds a `MockChain` where 3 notes are consumed by chained transactions in the same block.
    ///
    /// Returns the chain, the account, and the 3 notes (in consumption order).
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
        chain.prove_next_block().unwrap(); // block 1: makes genesis notes consumable

        // Execute 3 chained consume transactions (state S0→S1→S2→S3).
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

        chain.prove_next_block().unwrap(); // block 2: all 3 txs in one block
        (chain, account, [note1, note2, note3])
    }

    /// Verifies that `consumed_tx_order` is correctly set when multiple chained transactions
    /// for the same account consume notes in the same block.
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
    }

    #[tokio::test]
    async fn sync_state_across_multiple_iterations_with_same_mmr() {
        // Setup: create a mock chain and advance it so there are blocks to sync.
        let mock_rpc = MockRpcApi::default();
        mock_rpc.advance_blocks(3);
        let chain_tip_1 = mock_rpc.get_chain_tip_block_num();

        let state_sync = StateSync::new(Arc::new(mock_rpc.clone()), Arc::new(MockScreener), None);

        // Build the initial PartialMmr from genesis (only 1 leaf).
        let genesis_peaks = mock_rpc.get_mmr().peaks_at(Forest::new(1)).unwrap();
        let mut partial_mmr = PartialMmr::from_peaks(genesis_peaks);
        assert_eq!(partial_mmr.forest().num_leaves(), 1);

        // First sync
        let update = state_sync.sync_state(&mut partial_mmr, empty()).await.unwrap();

        assert_eq!(update.block_num, chain_tip_1);
        let forest_1 = partial_mmr.forest();
        // The MMR should contain one leaf per block (genesis + the new blocks).
        assert_eq!(forest_1.num_leaves(), chain_tip_1.as_u32() as usize + 1);

        // Second sync
        mock_rpc.advance_blocks(2);
        let chain_tip_2 = mock_rpc.get_chain_tip_block_num();

        let update = state_sync.sync_state(&mut partial_mmr, empty()).await.unwrap();

        assert_eq!(update.block_num, chain_tip_2);
        let forest_2 = partial_mmr.forest();
        assert!(forest_2 > forest_1);
        assert_eq!(forest_2.num_leaves(), chain_tip_2.as_u32() as usize + 1);

        // Third sync (no new blocks)
        let update = state_sync.sync_state(&mut partial_mmr, empty()).await.unwrap();

        assert_eq!(update.block_num, chain_tip_2);
        assert_eq!(partial_mmr.forest(), forest_2);
    }
}
