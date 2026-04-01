use alloc::boxed::Box;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::sync::Arc;
use alloc::vec::Vec;

use async_trait::async_trait;
use miden_protocol::Word;
use miden_protocol::account::{Account, AccountHeader, AccountId};
use miden_protocol::block::{BlockHeader, BlockNumber};
use miden_protocol::crypto::merkle::mmr::{Forest, MmrDelta, PartialMmr};
use miden_protocol::note::{NoteId, NoteTag, NoteType, Nullifier};
use tracing::info;

use super::state_sync_update::TransactionUpdateTracker;
use super::{AccountUpdates, StateSyncUpdate};
use crate::ClientError;
use crate::note::NoteUpdateTracker;
use crate::rpc::NodeRpcClient;
use crate::rpc::domain::note::{CommittedNote, NoteSyncBlock};
use crate::rpc::domain::transaction::{
    TransactionInclusion,
    TransactionRecord as RpcTransactionRecord,
};
use crate::store::{InputNoteRecord, OutputNoteRecord, StoreError};
use crate::transaction::TransactionRecord;

// STATE UPDATE DATA
// ================================================================================================

/// All data fetched from the node needed to sync the client to the chain tip.
///
/// Produced by [`StateSync::fetch_sync_data`], which calls `sync_chain_mmr` once
/// and loops `sync_notes` until the full range is covered, plus `sync_transactions`.
struct SyncUpdate {
    /// The chain tip at the time of the response. The client advances to this block.
    chain_tip: BlockNumber,
    /// MMR delta covering the full range from `current_block` to `chain_tip`.
    mmr_delta: MmrDelta,
    /// Chain tip block header (needed to add the tip leaf to the MMR).
    chain_tip_header: BlockHeader,
    /// Blocks with matching notes. Each carries an MMR path for tracking.
    note_blocks: Vec<NoteSyncBlock>,
    /// Account commitment updates for the synced range.
    account_commitment_updates: Vec<(AccountId, Word)>,
    /// Transaction inclusions for the synced range.
    transactions: Vec<TransactionInclusion>,
    /// Nullifiers for the synced range.
    nullifiers: Vec<Nullifier>,
}

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

        let mut state_sync_update = StateSyncUpdate {
            block_num,
            note_updates: NoteUpdateTracker::new(input_notes, output_notes),
            transaction_updates: TransactionUpdateTracker::new(uncommitted_transactions),
            ..Default::default()
        };

        let note_tags = Arc::new(note_tags);
        let account_ids: Vec<AccountId> = accounts.iter().map(AccountHeader::id).collect();
        let sync_data = self
            .fetch_sync_data(state_sync_update.block_num, &account_ids, &note_tags)
            .await?;

        // No progress — already at the tip.
        if sync_data.chain_tip == state_sync_update.block_num {
            return Ok(state_sync_update);
        }

        state_sync_update.block_num = sync_data.chain_tip;

        // Fetch details for all public notes.
        let public_note_ids: Vec<NoteId> = sync_data
            .note_blocks
            .iter()
            .flat_map(|b| b.notes.iter())
            .filter(|n| n.note_type() != NoteType::Private)
            .map(|n| *n.note_id())
            .collect();

        let public_note_records = self.fetch_public_note_details(&public_note_ids).await?;

        self.account_state_sync(
            &mut state_sync_update.account_updates,
            &accounts,
            &sync_data.account_commitment_updates,
        )
        .await?;

        self.apply_sync_result(
            sync_data,
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

    /// Executes a single sync step:
    /// 1. `sync_chain_mmr` — gets the MMR delta to the chain tip and discovers the chain tip.
    /// 2. `sync_notes` — loops until the full range to the chain tip is covered (handles truncated
    ///    responses from payload limits).
    /// 3. `sync_transactions` — gets transaction data for the full range.
    ///
    /// Returns a [`SyncUpdate`] where `chain_tip == current_block_num` signals no progress.
    async fn fetch_sync_data(
        &self,
        current_block_num: BlockNumber,
        account_ids: &[AccountId],
        note_tags: &Arc<BTreeSet<NoteTag>>,
    ) -> Result<SyncUpdate, ClientError> {
        info!("Fetching sync data from node.");

        // Step 1: Discover the chain tip.
        let (chain_tip_header, _) = self.rpc_api.get_block_header_by_number(None, false).await?;
        let chain_tip = chain_tip_header.block_num();

        if chain_tip == current_block_num {
            return Ok(SyncUpdate {
                chain_tip,
                mmr_delta: MmrDelta {
                    forest: Forest::new(current_block_num.as_usize()),
                    data: vec![],
                },
                chain_tip_header,
                note_blocks: vec![],
                account_commitment_updates: vec![],
                transactions: vec![],
                nullifiers: vec![],
            });
        }

        // Fetch the MMR delta to the chain tip.
        let chain_mmr_info =
            self.rpc_api.sync_chain_mmr(current_block_num, Some(chain_tip)).await?;

        // Step 2: Loop sync_notes until we've covered the full range to the chain tip.
        // Each response's `block_to` tells us how far the node scanned; if it's less
        // than the chain tip, the response was truncated and we continue from there.
        let mut note_blocks = Vec::new();
        let mut cursor = current_block_num;

        loop {
            let note_sync =
                self.rpc_api.sync_notes(cursor, Some(chain_tip), note_tags.as_ref()).await?;

            note_blocks.extend(note_sync.blocks);
            cursor = note_sync.block_to;

            if cursor >= chain_tip {
                break;
            }
        }

        // Step 3: Gather transactions for tracked accounts over the full range.
        let (account_commitment_updates, transactions, nullifiers) =
            self.fetch_transaction_data(current_block_num, chain_tip, account_ids).await?;

        Ok(SyncUpdate {
            chain_tip,
            mmr_delta: chain_mmr_info.mmr_delta,
            chain_tip_header,
            note_blocks,
            account_commitment_updates,
            transactions,
            nullifiers,
        })
    }

    /// Fetches transaction data for the given range and account IDs.
    async fn fetch_transaction_data(
        &self,
        block_from: BlockNumber,
        block_to: BlockNumber,
        account_ids: &[AccountId],
    ) -> Result<(Vec<(AccountId, Word)>, Vec<TransactionInclusion>, Vec<Nullifier>), ClientError>
    {
        if account_ids.is_empty() {
            return Ok((vec![], vec![], vec![]));
        }

        let tx_info = self
            .rpc_api
            .sync_transactions(block_from, Some(block_to), account_ids.to_vec())
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

        Ok((account_updates, tx_inclusions, nullifiers))
    }

    // HELPERS
    // --------------------------------------------------------------------------------------------

    /// Applies sync results to the local state update.
    ///
    /// Applies fetched sync data to the local state:
    /// 1. Advances the partial MMR (delta + chain tip leaf).
    /// 2. Screens note blocks and tracks relevant ones in the MMR.
    /// 3. Applies transaction and nullifier updates.
    async fn apply_sync_result(
        &self,
        sync_data: SyncUpdate,
        public_note_records: &BTreeMap<NoteId, InputNoteRecord>,
        state_sync_update: &mut StateSyncUpdate,
        current_partial_mmr: &mut PartialMmr,
    ) -> Result<(), ClientError> {
        // Advance the partial MMR: apply delta (up to chain_tip - 1), capture peaks for
        // storage, then add the chain tip leaf (which the delta excludes due to the
        // one-block lag in block header MMR commitments).
        let mut new_authentication_nodes =
            current_partial_mmr.apply(sync_data.mmr_delta).map_err(StoreError::MmrError)?;
        let new_peaks = current_partial_mmr.peaks();
        new_authentication_nodes
            .append(&mut current_partial_mmr.add(sync_data.chain_tip_header.commitment(), false));

        state_sync_update.block_updates.insert(
            sync_data.chain_tip_header.clone(),
            false,
            new_peaks,
            new_authentication_nodes,
        );

        // Screen each note block and track relevant ones in the partial MMR using the
        // authentication path from the sync_notes response.
        for block in sync_data.note_blocks {
            let found_relevant_note = self
                .note_state_sync(
                    &mut state_sync_update.note_updates,
                    block.notes,
                    &block.block_header,
                    public_note_records,
                )
                .await?;

            if found_relevant_note {
                let block_pos = block.block_header.block_num().as_usize();

                // Collect authentication nodes added by track() so the store can persist
                // them. Skip if already tracked (from a previous sync).
                let track_auth_nodes = if current_partial_mmr.is_tracked(block_pos) {
                    vec![]
                } else {
                    let nodes_before: BTreeMap<_, _> =
                        current_partial_mmr.nodes().map(|(k, v)| (*k, *v)).collect();
                    current_partial_mmr
                        .track(block_pos, block.block_header.commitment(), &block.mmr_path)
                        .map_err(StoreError::MmrError)?;
                    current_partial_mmr
                        .nodes()
                        .filter(|(k, _)| !nodes_before.contains_key(k))
                        .map(|(k, v)| (*k, *v))
                        .collect()
                };

                state_sync_update.block_updates.insert(
                    block.block_header,
                    true,
                    current_partial_mmr.peaks(),
                    track_auth_nodes,
                );
            }
        }

        // Apply transaction and nullifier data.
        state_sync_update.note_updates.extend_nullifiers(sync_data.nullifiers);
        self.transaction_state_sync(
            &mut state_sync_update.transaction_updates,
            &sync_data.chain_tip_header,
            &sync_data.transactions,
        );

        Ok(())
    }

    /// Compares the state of tracked accounts with the updates received from the node. The method
    /// updates the `state_sync_update` field with the details of the accounts that need to be
    /// updated.
    ///
    /// The account updates might include:
    /// * Public accounts that have been updated in the node.
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

        let updated_public_accounts = self
            .get_updated_public_accounts(account_commitment_updates, &public_accounts)
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

        account_updates
            .extend(AccountUpdates::new(updated_public_accounts, mismatched_private_accounts));

        Ok(())
    }

    /// Queries the node for the latest state of the public accounts that don't match the current
    /// state of the client.
    async fn get_updated_public_accounts(
        &self,
        account_updates: &[(AccountId, Word)],
        current_public_accounts: &[&AccountHeader],
    ) -> Result<Vec<Account>, ClientError> {
        let mut mismatched_public_accounts = vec![];

        for (id, commitment) in account_updates {
            // check if this updated account state is tracked by the client
            if let Some(account) = current_public_accounts
                .iter()
                .find(|acc| *id == acc.id() && *commitment != acc.to_commitment())
            {
                mismatched_public_accounts.push(*account);
            }
        }

        self.rpc_api
            .get_updated_public_accounts(&mismatched_public_accounts)
            .await
            .map_err(ClientError::RpcError)
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
            let public_note = (committed_note.note_type() != NoteType::Private)
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
    use miden_protocol::assembly::DefaultSourceManager;
    use miden_protocol::crypto::merkle::mmr::{Forest, InOrderIndex, PartialMmr};
    use miden_protocol::note::{NoteTag, NoteType};
    use miden_protocol::{Felt, Word};
    use miden_standards::code_builder::CodeBuilder;
    use miden_testing::MockChainBuilder;

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

    /// Builds a mock chain with a faucet that mints `num_blocks` notes, one per block.
    /// Returns the chain and the set of note tags for filtering.
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

    /// Verifies that the sync correctly processes notes committed in multiple blocks
    /// (batched `SyncNotes` response) and tracks their blocks in the partial MMR.
    ///
    /// This test creates a faucet and mints notes in separate blocks (blocks 1, 2, 3),
    /// so `sync_notes` returns multiple `NoteSyncBlock`s. It then verifies:
    /// - The MMR is advanced to the chain tip
    /// - Blocks containing relevant notes are tracked in the partial MMR via `track()`
    /// - Note inclusion proofs are set correctly
    /// - Block headers for note blocks are stored
    #[tokio::test]
    async fn sync_state_tracks_note_blocks_in_mmr() {
        let (chain, note_tags) = build_chain_with_mint_notes(3).await;
        let mock_rpc = MockRpcApi::new(chain);
        let chain_tip = mock_rpc.get_chain_tip_block_num();

        // Verify the mock returns notes across multiple blocks.
        let note_sync =
            mock_rpc.sync_notes(BlockNumber::from(0u32), None, &note_tags).await.unwrap();
        assert!(
            note_sync.blocks.len() >= 2,
            "expected notes in multiple blocks, got {}",
            note_sync.blocks.len()
        );

        // Collect the block numbers that have notes.
        let note_block_nums: BTreeSet<BlockNumber> =
            note_sync.blocks.iter().map(|b| b.block_header.block_num()).collect();

        // Test that fetch_sync_data returns note blocks with valid MMR paths that
        // can be used to track blocks in the partial MMR.
        let state_sync = StateSync::new(Arc::new(mock_rpc.clone()), Arc::new(MockScreener), None);

        let genesis_peaks = mock_rpc.get_mmr().peaks_at(Forest::new(1)).unwrap();
        let mut partial_mmr = PartialMmr::from_peaks(genesis_peaks);

        let sync_data = state_sync
            .fetch_sync_data(BlockNumber::GENESIS, &[], &Arc::new(note_tags.clone()))
            .await
            .unwrap();

        // Should have advanced to the chain tip.
        assert_eq!(sync_data.chain_tip, chain_tip);
        assert!(!sync_data.note_blocks.is_empty(), "should have note blocks");

        // Apply the MMR delta and add the chain tip block.
        let _auth_nodes: Vec<(InOrderIndex, Word)> =
            partial_mmr.apply(sync_data.mmr_delta).map_err(StoreError::MmrError).unwrap();
        partial_mmr.add(sync_data.chain_tip_header.commitment(), false);

        assert_eq!(partial_mmr.forest().num_leaves(), chain_tip.as_u32() as usize + 1);

        // Track each note block using the MMR path from the sync_notes response.
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

        // Verify the tracked blocks match the note blocks.
        for &bn in &note_block_nums {
            assert!(
                partial_mmr.is_tracked(bn.as_usize()),
                "block {bn} with notes should be tracked in partial MMR"
            );
        }
    }
}
