use alloc::boxed::Box;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::sync::Arc;
use alloc::vec::Vec;

use async_trait::async_trait;
use miden_protocol::Word;
use miden_protocol::account::{Account, AccountHeader, AccountId, StorageSlotType};
use miden_protocol::block::{BlockHeader, BlockNumber};
use miden_protocol::crypto::merkle::mmr::{MmrDelta, PartialMmr};
use miden_protocol::note::{NoteAttachments, NoteId, NoteTag, NoteType, Nullifier};
use tracing::{info, warn};

use super::state_sync_update::TransactionUpdateTracker;
use super::{
    AccountUpdates,
    PartialBlockchainUpdates,
    PublicAccountDelta,
    PublicAccountUpdate,
    StateSyncUpdate,
};
use crate::ClientError;
use crate::note::{NoteConsumption, NoteUpdateTracker};
use crate::rpc::NodeRpcClient;
use crate::rpc::domain::account::{AccountDetails, GetAccountRequest, StorageMapFetch, VaultFetch};
use crate::rpc::domain::note::{CommittedNote, NoteSyncBlock, SyncedNoteDetails};
use crate::rpc::domain::sync::{ChainMmrInfo, SyncTarget};
use crate::rpc::domain::transaction::TransactionRecord as RpcTransactionRecord;
use crate::store::{InputNoteRecord, OutputNoteRecord, StoreError};
use crate::transaction::TransactionRecord;

// STATE UPDATE DATA
// ================================================================================================

/// Data fetched from the node needed to sync the client to the chain tip.
///
/// Aggregates the responses of `sync_chain_mmr`, `sync_notes`, `get_notes_by_id`, and
/// `sync_transactions`. This may contain more data than a particular client needs to store — it is
/// filtered and transformed into a [`StateSyncUpdate`] before being applied.
struct FetchedSyncData {
    /// MMR delta covering the full range from `current_block` to `chain_tip`.
    mmr_delta: MmrDelta,
    /// Chain tip block header.
    chain_tip_header: BlockHeader,
    /// Blocks with matching notes that the client is interested in.
    note_blocks: Vec<NoteSyncBlock>,
    /// Content fetched for the synced notes (public note bodies and private-note attachments),
    /// keyed by note ID.
    synced_notes: BTreeMap<NoteId, SyncedNoteDetails>,
    /// Transaction records for the synced range, as returned by `sync_transactions`.
    transactions: Vec<RpcTransactionRecord>,
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
    /// Headers of the tracked accounts to follow during the sync.
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
    /// 1. Fetch sync data from the node (MMR delta, note inclusions, transactions).
    /// 2. Update account states (fetch updated public accounts, flag mismatched private ones).
    /// 3. Advance the partial MMR to the chain tip.
    /// 4. Screen note inclusions via the configured [`OnNoteReceived`] callback and track relevant
    ///    blocks in the MMR.
    /// 5. Process transaction inclusions (commit local txs, record external consumers, discard
    ///    stale/expired txs, commit output notes).
    /// 6. Detect consumed notes via nullifier sync (optional, see
    ///    [`Self::disable_nullifier_sync`]).
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

        let note_tags = Arc::new(note_tags);
        let account_ids: Vec<AccountId> = accounts.iter().map(AccountHeader::id).collect();

        // Account states in the lineage of in-flight local transactions (each transaction's
        // initial and final state), used to recognize on-chain states that the local state
        // already builds on.
        let mut in_flight_account_states: BTreeMap<AccountId, Vec<Word>> = BTreeMap::new();
        for tx in &uncommitted_transactions {
            let states = in_flight_account_states.entry(tx.details.account_id).or_default();
            states.push(tx.details.init_account_state);
            states.push(tx.details.final_account_state);
        }

        let mut state_sync_update = StateSyncUpdate {
            block_num,
            note_updates: NoteUpdateTracker::new(input_notes, output_notes),
            transaction_updates: TransactionUpdateTracker::new(uncommitted_transactions),
            ..Default::default()
        };
        let Some(sync_data) = self
            .fetch_sync_data(state_sync_update.block_num, &account_ids, &note_tags)
            .await?
        else {
            // No progress — already at the tip.
            return Ok(state_sync_update);
        };

        state_sync_update.block_num = sync_data.chain_tip_header.block_num();
            let acc_tx_by_block =
                AccountTransactionsByBlock::from(sync_data.transactions.as_slice());
        let new_commitments =  acc_tx_by_block.get_account_commitments();
        let ordered_nullifiers = acc_tx_by_block.ordered_nullifiers();

        self.account_state_sync(
            &mut state_sync_update.account_updates,
            &accounts,
            &new_commitments,
            &in_flight_account_states,
            block_num,
        )
        .await?;

        // Apply local changes: update the MMR, screen notes, and apply state transitions.
        self.apply_sync_result(
            sync_data,
            ordered_nullifiers,
            &mut state_sync_update,
            current_partial_mmr,
        )
        .await?;

        if self.sync_nullifiers {
            self.nullifiers_state_sync(&mut state_sync_update, block_num).await?;
        }

        Ok(state_sync_update)
    }

    /// Fetches the sync data from the node by calling the following endpoints:
    /// 1. `sync_chain_mmr` — discovers the chain tip, gets the MMR delta and chain tip header.
    /// 2. `sync_notes` — loops until the full range to the chain tip is covered (handles paginated
    ///    responses).
    /// 3. `get_notes_by_id` — fetches full metadata for notes with attachments.
    /// 4. `sync_transactions` — gets transaction data for the full range.
    ///
    /// Returns `None` when the client is already at the chain tip (no progress).
    async fn fetch_sync_data(
        &self,
        current_block_num: BlockNumber,
        account_ids: &[AccountId],
        note_tags: &Arc<BTreeSet<NoteTag>>,
    ) -> Result<Option<FetchedSyncData>, ClientError> {
        // Step 1: Fetch the MMR delta and chain tip header.
        let chain_mmr_info = self
            .rpc_api
            .sync_chain_mmr(current_block_num, SyncTarget::CommittedChainTip)
            .await?;
        let chain_tip = chain_mmr_info.block_to;

        // Validate the response covers the range we requested.
        Self::validate_chain_mmr_response(&chain_mmr_info, current_block_num)?;

        // No progress — already at the tip.
        if chain_tip == current_block_num {
            info!(block_num = %current_block_num, "Already at chain tip, nothing to sync.");
            return Ok(None);
        }

        info!(
            block_from = %current_block_num,
            block_to = %chain_tip,
            "Syncing state.",
        );

        // Step 2: sync notes and fetch full note bodies for public notes (and attachment content
        // for private notes that carry attachments), paginating with the same chain tip so MMR
        // paths are opened at a consistent forest. With no tracked tags there's nothing the node
        // could match, so skip the RPC entirely.
        let (note_blocks, synced_notes) = if note_tags.is_empty() {
            (Vec::new(), BTreeMap::new())
        } else {
            self.rpc_api
                .sync_notes_with_details(current_block_num + 1, chain_tip, note_tags.as_ref())
                .await?
        };

        // Validate every returned note block falls in (current_block_num, chain_tip].
        Self::validate_note_blocks_range(&note_blocks, current_block_num, chain_tip)?;

        let note_count: usize = note_blocks.iter().map(|b| b.notes.len()).sum();
        info!(
            blocks_with_notes = note_blocks.len(),
            notes = note_count,
            synced_notes = synced_notes.len(),
            "Fetched note sync data.",
        );

        // Step 3: sync transactions for tracked accounts over the full range. With no tracked
        // accounts there's nothing the node could match, so skip the RPC entirely.
        let transaction_records = if account_ids.is_empty() {
            Vec::new()
        } else {
            self.rpc_api
                .sync_transactions(current_block_num + 1, chain_tip, account_ids.to_vec())
                .await?
        };

        Ok(Some(FetchedSyncData {
            mmr_delta: chain_mmr_info.mmr_delta,
            chain_tip_header: chain_mmr_info.block_header,
            note_blocks,
            synced_notes,
            transactions: transaction_records,
        }))
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
        sync_data: FetchedSyncData,
        ordered_nullifiers: Vec<Nullifier>,
        state_sync_update: &mut StateSyncUpdate,
        current_partial_mmr: &mut PartialMmr,
    ) -> Result<(), ClientError> {
        let FetchedSyncData {
            mmr_delta,
            chain_tip_header,
            note_blocks,
            synced_notes,
            transactions,
        } = sync_data;

        // Operate on a clone so any validation failure leaves `current_partial_mmr` untouched.
        // The clone is committed back at the end of the function once all checks pass.
        let mut working_mmr = current_partial_mmr.clone();

        state_sync_update.partial_blockchain_updates =
            Self::advance_mmr(mmr_delta, &chain_tip_header, &mut working_mmr)?;

        self.screen_note_blocks(note_blocks, synced_notes, state_sync_update, &mut working_mmr)
            .await?;

        self.apply_transactions_and_nullifiers(
            &chain_tip_header,
            &transactions,
            ordered_nullifiers,
            state_sync_update,
        )?;

        // Commit the working MMR back to the caller once all checks pass.
        *current_partial_mmr = working_mmr;

        Ok(())
    }

    /// Validates that a `sync_chain_mmr` response covers the requested range.
    fn validate_chain_mmr_response(
        chain_mmr_info: &ChainMmrInfo,
        current_block_num: BlockNumber,
    ) -> Result<(), ClientError> {
        if chain_mmr_info.block_header.block_num() != chain_mmr_info.block_to {
            return Err(ClientError::ChainValidationError(format!(
                "sync_chain_mmr block_header.block_num ({}) does not match block_to ({})",
                chain_mmr_info.block_header.block_num(),
                chain_mmr_info.block_to
            )));
        }
        if chain_mmr_info.block_from != current_block_num {
            return Err(ClientError::ChainValidationError(format!(
                "sync_chain_mmr block_from mismatch: expected {current_block_num}, got {}",
                chain_mmr_info.block_from
            )));
        }
        if chain_mmr_info.block_to < current_block_num {
            return Err(ClientError::ChainValidationError(format!(
                "sync_chain_mmr block_to ({}) is behind current block {current_block_num}",
                chain_mmr_info.block_to
            )));
        }
        Ok(())
    }

    /// Validates that every block returned by `sync_notes` falls in the requested range
    /// `(current_block_num, chain_tip]`.
    fn validate_note_blocks_range(
        note_blocks: &[NoteSyncBlock],
        current_block_num: BlockNumber,
        chain_tip: BlockNumber,
    ) -> Result<(), ClientError> {
        for block in note_blocks {
            let block_num = block.block_header.block_num();
            if block_num <= current_block_num || block_num > chain_tip {
                return Err(ClientError::ChainValidationError(format!(
                    "sync_notes returned block {block_num} outside requested range ({current_block_num}, {chain_tip}]"
                )));
            }
        }
        Ok(())
    }

    /// Applies the MMR delta and adds the chain-tip leaf, returning the resulting partial
    /// blockchain updates (new peaks, the chain-tip header and its authentication nodes). The
    /// delta excludes the chain-tip leaf because of the one-block lag in block header MMR
    /// commitments, so the tip leaf has to be added separately.
    ///
    /// Before adding the chain-tip leaf, the post-delta peaks are checked against the chain
    /// tip header's chain commitment to ensure the delta advanced the MMR to the expected state.
    fn advance_mmr(
        mmr_delta: MmrDelta,
        chain_tip_header: &BlockHeader,
        current_partial_mmr: &mut PartialMmr,
    ) -> Result<PartialBlockchainUpdates, ClientError> {
        let mut new_authentication_nodes =
            current_partial_mmr.apply(mmr_delta).map_err(StoreError::MmrError)?;
        let new_peaks = current_partial_mmr.peaks();

        // Verify that post-delta peaks match the block header's chain commitment.
        // chain_commitment is the hash of MMR peaks for blocks 0..block_num-1,
        // which is exactly the state after applying the delta.
        let peaks_commitment = new_peaks.hash_peaks();
        if peaks_commitment != chain_tip_header.chain_commitment() {
            return Err(ClientError::ChainValidationError(format!(
                "MMR peaks commitment is {} and does not match block header chain commitment {}",
                peaks_commitment.to_hex(),
                chain_tip_header.chain_commitment().to_hex()
            )));
        }

        // Note: we add the chain tip leaf to our MMR, but we cannot prove that it is effectively
        // the chain tip. In the current context of centralized trusted node, we assume it
        // is valid. Eventually, we will be able to validate that the resulting MMR root is
        // "canonical".
        new_authentication_nodes.append(
            &mut current_partial_mmr
                .add(chain_tip_header.commitment(), false)
                .map_err(StoreError::MmrError)?,
        );

        let mut partial_blockchain_updates = PartialBlockchainUpdates::new(new_peaks);
        partial_blockchain_updates.insert(
            chain_tip_header.clone(),
            false,
            new_authentication_nodes,
        );

        Ok(partial_blockchain_updates)
    }

    /// Screens each note block for relevance and, for blocks containing client-relevant notes,
    /// tracks them in the partial MMR using the authentication path from the `sync_notes`
    /// response.
    async fn screen_note_blocks(
        &self,
        note_blocks: Vec<NoteSyncBlock>,
        synced_notes: BTreeMap<NoteId, SyncedNoteDetails>,
        state_sync_update: &mut StateSyncUpdate,
        current_partial_mmr: &mut PartialMmr,
    ) -> Result<(), ClientError> {
        // Attachment content for private notes, keyed by note ID. Joined to each committed note
        // by ID so the stored record reconstructs the correct note ID.
        let private_attachments: BTreeMap<NoteId, NoteAttachments> = synced_notes
            .iter()
            .filter_map(|(id, synced)| match synced {
                SyncedNoteDetails::Private(Some(attachments)) => Some((*id, attachments.clone())),
                _ => None,
            })
            .collect();
        let public_note_records = Self::build_public_note_records(synced_notes, &note_blocks);

        for block in note_blocks {
            let found_relevant_note = self
                .note_state_sync(
                    &mut state_sync_update.note_updates,
                    block.notes,
                    &block.block_header,
                    &public_note_records,
                    &private_attachments,
                )
                .await?;

            if found_relevant_note {
                let block_pos = block.block_header.block_num().as_usize();

                let nodes_before: BTreeMap<_, _> =
                    current_partial_mmr.nodes().map(|(k, v)| (*k, *v)).collect();

                if !current_partial_mmr.is_tracked(block_pos) {
                    current_partial_mmr
                        .track(block_pos, block.block_header.commitment(), &block.mmr_path)
                        .map_err(StoreError::MmrError)?;
                }

                // Always collect new authentication nodes — even when the block was
                // already tracked from the MMR delta, the delta's nodes may not include
                // the full authentication path needed to reconstruct the PartialMmr
                // from storage later.
                let track_auth_nodes: Vec<_> = current_partial_mmr
                    .nodes()
                    .filter(|(k, _)| !nodes_before.contains_key(k))
                    .map(|(k, v)| (*k, *v))
                    .collect();

                state_sync_update.partial_blockchain_updates.insert(
                    block.block_header,
                    true,
                    track_auth_nodes,
                );
            }
        }

        Ok(())
    }

    /// Extends the note tracker with newly-observed nullifiers, applies transaction
    /// inclusions, and walks each transaction to apply output-note inclusion proofs and mark
    /// same-batch-erased output notes as consumed.
    fn apply_transactions_and_nullifiers(
        &self,
        chain_tip_header: &BlockHeader,
        transactions: &[RpcTransactionRecord],
        ordered_nullifiers: Vec<Nullifier>,
        state_sync_update: &mut StateSyncUpdate,
    ) -> Result<(), ClientError> {
        state_sync_update.note_updates.extend_nullifiers(ordered_nullifiers);

        for record in transactions {
            state_sync_update
                .transaction_updates
                .apply_transaction_inclusion(record, u64::from(chain_tip_header.timestamp())); //TODO: Change timestamps from u64 to u32
        }
        state_sync_update
            .transaction_updates
            .apply_sync_height_update(chain_tip_header.block_num(), self.tx_discard_delta);

        for transaction in transactions {
            // Transition tracked output notes to Committed using inclusion proofs from the
            // transaction sync response. This covers output notes regardless of whether their
            // tags were tracked in the note sync.
            state_sync_update
                .note_updates
                .apply_output_note_inclusion_proofs(&transaction.output_notes)?;

            // Detect output notes erased by same-batch note erasure.
            Self::mark_erased_notes_as_consumed(state_sync_update, transaction);
        }

        Ok(())
    }

    /// Marks output notes that were erased by same-batch note erasure as consumed.
    ///
    /// When a note is created and consumed in the same batch, note erasure removes it from
    /// the block body. The node reports these as erased output notes in the transaction
    /// record (note ID only, no inclusion proof). We mark them as consumed.
    fn mark_erased_notes_as_consumed(
        state_sync_update: &mut StateSyncUpdate,
        transaction: &RpcTransactionRecord,
    ) {
        for note_header in &transaction.erased_output_notes {
            // Best-effort: ignore errors for notes not tracked by this client.
            let _ = state_sync_update
                .note_updates
                .mark_erased_note_as_consumed(note_header, transaction.block_num);
        }
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
        account_commitment_updates: &BTreeMap<AccountId, Word>,
        in_flight_account_states: &BTreeMap<AccountId, Vec<Word>>,
        block_from: BlockNumber,
    ) -> Result<(), ClientError> {
        // "Public" here includes both Public and Network accounts, since both have
        // their state stored on-chain and follow the same sync path.
        let (public_accounts, private_accounts): (Vec<_>, Vec<_>) =
            accounts.iter().partition(|header| !header.id().is_private());

        self.sync_public_accounts(
            account_updates,
            account_commitment_updates,
            in_flight_account_states,
            &public_accounts,
            block_from,
        )
        .await?;

        let mismatched_private_accounts = account_commitment_updates
            .iter()
            .filter(|&(account_id, digest)| {
                private_accounts
                    .iter()
                    .any(|header| header.id() == *account_id && &header.to_commitment() != digest)
            })
            .map(|(account_id, digest)| (*account_id, *digest))
            .collect::<Vec<_>>();

        account_updates.extend(AccountUpdates::new(Vec::new(), mismatched_private_accounts));

        Ok(())
    }

    /// Queries the node for updated public accounts and populates `account_updates`.
    ///
    /// For each public account whose commitment changed, an updated snapshot is fetched with a
    /// single `get_account` call that requests every storage map and the vault.
    ///
    /// Accounts whose vault or maps are too large to fit in a single response fall back to the
    /// incremental [`PublicAccountUpdate::Delta`] path, which fetches vault and storage map
    /// updates over the synced block range.
    async fn sync_public_accounts(
        &self,
        account_updates: &mut AccountUpdates,
        commitment_updates: &BTreeMap<AccountId, Word>,
        in_flight_account_states: &BTreeMap<AccountId, Vec<Word>>,
        current_public_accounts: &[&AccountHeader],
        block_from: BlockNumber,
    ) -> Result<(), ClientError> {
        let local_states: BTreeMap<AccountId, (Word, u64)> = current_public_accounts
            .iter()
            .map(|header| {
                (header.id(), (header.to_commitment(), header.nonce().as_canonical_u64()))
            })
            .collect();
        for (id, commitment) in commitment_updates {
            let Some((local_commitment, local_nonce)) = local_states.get(id) else {
                continue;
            };
            if local_commitment == commitment {
                continue;
            }

            let public_update = self.sync_public_account(*id, block_from).await?;
            let fetched_commitment = public_update.commitment();

            // An on-chain state matching the local one or a state in the in-flight
            // transaction lineage is a state the local one already builds on: the chain
            // simply lags while a submitted transaction is in flight. Keep the local state
            // and reconcile on a later sync.
            if fetched_commitment == *local_commitment
                || in_flight_account_states
                    .get(id)
                    .is_some_and(|states| states.contains(&fetched_commitment))
            {
                continue;
            }

            // An on-chain state outside the in-flight lineage with an older nonce means the
            // local state diverged from the chain (e.g. a conflicting transaction submitted
            // elsewhere was committed). Applying it would roll the stored nonce back, which
            // the store rejects, so skip it; transaction discarding will reconcile the
            // account on a later sync.
            if public_update.nonce().as_canonical_u64() < *local_nonce {
                warn!(
                    account_id = %id,
                    "on-chain account state is older than the local one and outside the \
                     in-flight transaction lineage; skipping until the local state is \
                     reconciled"
                );
                continue;
            }

            account_updates.extend(AccountUpdates::new(vec![public_update], Vec::new()));
        }

        Ok(())
    }

    /// Fetches an updated snapshot for a single public account.
    ///
    /// Must only be called when the local commitment for the account is known to differ from the
    /// network's. Note that the returned state may still lag the local one (e.g. while a
    /// locally-submitted transaction is in flight), so the caller must check the returned
    /// commitment against the local state's lineage before applying it.
    ///
    /// # Panics
    ///
    /// Panics if the node response omits account details, since that would mean the account is
    /// not public.
    async fn sync_public_account(
        &self,
        account_id: AccountId,
        block_from: BlockNumber,
    ) -> Result<PublicAccountUpdate, ClientError> {
        // A single request fetches the full snapshot: every storage map's entries plus the vault,
        // with the storage layout discovered server-side.
        let (proof_block_num, proof) = self
            .rpc_api
            .get_account(
                account_id,
                GetAccountRequest::new()
                    .with_storage(StorageMapFetch::All)
                    .with_vault(VaultFetch::Always),
            )
            .await
            .map_err(ClientError::RpcError)?;

        let details = proof.into_details().expect("node returned no details for a public account");

        let vault_oversized = details.vault_details.too_many_assets;
        let any_map_oversized =
            details.storage_details.map_details.iter().any(|m| m.too_many_entries);

        // TODO: we can handle vault and storage-map oversize independently. Today any oversize
        // routes the whole account through the incremental delta path, which always fetches
        // both `sync_storage_maps` and `sync_account_vault`, even if not needed.
        let public_update = if vault_oversized || any_map_oversized {
            // Some part of the account is oversized — use incremental endpoints.
            self.build_delta_update(account_id, &details, block_from, proof_block_num)
                .await?
        } else {
            // The single response carries the full vault and every map's entries.
            let account = Account::try_from(&details).map_err(ClientError::RpcError)?;
            PublicAccountUpdate::Full(account)
        };

        Ok(public_update)
    }

    /// Builds a [`PublicAccountUpdate::Delta`] by fetching incremental storage map and vault
    /// updates over the synced range.
    async fn build_delta_update(
        &self,
        account_id: AccountId,
        details: &AccountDetails,
        block_from: BlockNumber,
        block_to: BlockNumber,
    ) -> Result<PublicAccountUpdate, ClientError> {
        let value_slot_updates: Vec<(_, Word)> = details
            .storage_details
            .header
            .slots()
            .filter(|slot| slot.slot_type() == StorageSlotType::Value)
            .map(|slot| (slot.name().clone(), slot.value()))
            .collect();

        // The lower bound is inclusive at the node, so request from `block_from + 1` to skip
        // the block whose state we already have.
        let map_info = self
            .rpc_api
            .sync_storage_maps(block_from + 1, block_to, account_id)
            .await
            .map_err(ClientError::RpcError)?;
        let vault_info = self
            .rpc_api
            .sync_account_vault(block_from + 1, block_to, account_id)
            .await
            .map_err(ClientError::RpcError)?;

        Ok(PublicAccountUpdate::Delta(PublicAccountDelta::new(
            details.header.clone(),
            block_from,
            block_to,
            value_slot_updates,
            map_info.updates,
            vault_info.updates,
        )))
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
    /// iteration so the node is only queried once per batch. The `private_attachments` parameter
    /// carries attachment content resolved for private notes, keyed by note ID; it is joined to
    /// each committed note by ID so the stored record reconstructs the correct note ID.
    async fn note_state_sync(
        &self,
        note_updates: &mut NoteUpdateTracker,
        note_inclusions: BTreeMap<NoteId, CommittedNote>,
        block_header: &BlockHeader,
        public_notes: &BTreeMap<NoteId, InputNoteRecord>,
        private_attachments: &BTreeMap<NoteId, NoteAttachments>,
    ) -> Result<bool, ClientError> {
        // `found_relevant_note` tracks whether we want to persist the block header in the end
        let mut found_relevant_note = false;

        for (_, committed_note) in note_inclusions {
            let public_note = (committed_note.note_type() != NoteType::Private)
                .then(|| public_notes.get(committed_note.note_id()))
                .flatten()
                .cloned();

            match self.note_screener.on_note_received(committed_note, public_note).await? {
                NoteUpdateAction::Commit(committed_note) => {
                    // Only mark the downloaded block header as relevant if we are talking about
                    // an input note (output notes get marked as committed but we don't need the
                    // block for anything there)
                    let attachments = private_attachments.get(committed_note.note_id());
                    found_relevant_note |= note_updates.apply_committed_note_state_transitions(
                        &committed_note,
                        block_header,
                        attachments,
                    )?;
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
        // (it only returns nullifiers from current_block_num + 1 until state_sync_update.block_num)

        // Check for new nullifiers for input notes that were updated
        let nullifiers_tags: Vec<u16> = state_sync_update
            .note_updates
            .unspent_nullifiers()
            .map(|nullifier| nullifier.prefix())
            .collect();

        let mut new_nullifiers = self
            .rpc_api
            .sync_nullifiers(&nullifiers_tags, current_block_num + 1, state_sync_update.block_num)
            .await?;

        // Discard nullifiers that are newer than the current block (this might happen if the block
        // changes between the sync_state and the check_nullifier calls)
        new_nullifiers.retain(|update| update.block_num <= state_sync_update.block_num);

        // Match each nullifier update with the externally-tracked consumer account.
        let consumptions: Vec<NoteConsumption> = new_nullifiers
            .into_iter()
            .map(|update| NoteConsumption {
                external_consumer: state_sync_update
                    .transaction_updates
                    .external_nullifier_account(&update.nullifier),
                nullifier: update.nullifier,
                block_num: update.block_num,
            })
            .collect();

        for consumption in consumptions {
            state_sync_update.note_updates.apply_note_consumption(
                &consumption,
                state_sync_update.transaction_updates.committed_transactions(),
            )?;

            // Process nullifiers and track the updates of local tracked transactions that were
            // discarded because the notes that they were processing were nullified by an
            // another transaction.
            state_sync_update
                .transaction_updates
                .apply_input_note_nullified(consumption.nullifier);
        }

        Ok(())
    }

    /// Pairs each public note body with the matching inclusion proof from `note_blocks`. Private
    /// notes and public notes without a matching inclusion proof are dropped.
    fn build_public_note_records(
        synced_notes: BTreeMap<NoteId, SyncedNoteDetails>,
        note_blocks: &[NoteSyncBlock],
    ) -> BTreeMap<NoteId, InputNoteRecord> {
        let mut records = BTreeMap::new();
        for (note_id, synced) in synced_notes {
            let SyncedNoteDetails::Public(note) = synced else {
                continue;
            };
            let inclusion_proof = note_blocks
                .iter()
                .find_map(|b| b.notes.get(&note_id))
                .map(|committed| committed.inclusion_proof().clone());

            if let Some(inclusion_proof) = inclusion_proof {
                let state = crate::store::input_note_states::UnverifiedNoteState {
                    metadata: *note.metadata(),
                    inclusion_proof,
                }
                .into();
                let attachments = note.attachments().clone();
                let record = InputNoteRecord::new(note.into(), attachments, None, state);
                let id = record.id().expect("CommittedNoteState carries metadata, so id() is Some");
                records.insert(id, record);
            }
        }
        records
    }
}

// HELPERS
// ================================================================================================

/// Transaction records grouped by `(account_id, block_num)`.
struct AccountTransactionsByBlock<'a> {
    groups: BTreeMap<(AccountId, BlockNumber), Vec<&'a RpcTransactionRecord>>,
}

impl<'a> From<&'a [RpcTransactionRecord]> for AccountTransactionsByBlock<'a> {
    fn from(transaction_records: &'a [RpcTransactionRecord]) -> Self {
        let mut groups: BTreeMap<(AccountId, BlockNumber), Vec<&'a RpcTransactionRecord>> =
            BTreeMap::new();
        for record in transaction_records {
            let account_id = record.transaction_header.account_id();
            groups.entry((account_id, record.block_num)).or_default().push(record);
        }

        Self { groups }
    }
}

impl<'a> AccountTransactionsByBlock<'a> {
    fn iter(
        &self,
    ) -> impl Iterator<Item = (AccountId, BlockNumber, &[&'a RpcTransactionRecord])> + '_ {
        self.groups
            .iter()
            .map(|(&(account_id, block_num), txs)| (account_id, block_num, txs.as_slice()))
    }

    /// Derives account commitment updates from transaction records.
    ///
    /// For each unique account, returns the `final_state_commitment` from the final transaction
    /// with the highest `block_num`.
    fn get_account_commitments(&self) -> BTreeMap<AccountId, Word> {
        let mut latest_by_account: BTreeMap<AccountId, (BlockNumber, Word)> = BTreeMap::new();

        for (account_id, block_num, txs) in self.iter() {
            let terminal_state = walk_execution_chain(txs)
                .last()
                .expect("account must have a final state")
                .transaction_header
                .final_state_commitment();

            latest_by_account
                .entry(account_id)
                .and_modify(|(existing_block, existing_state)| {
                    if block_num > *existing_block {
                        *existing_block = block_num;
                        *existing_state = terminal_state;
                    }
                })
                .or_insert((block_num, terminal_state));
        }

        latest_by_account
            .into_iter()
            .map(|(account_id, (_, state))| (account_id, state))
            .collect()
    }

    /// Returns nullifiers ordered by consuming transaction position, per account.
    ///
    /// Groups RPC transaction records by (`account_id`, `block_num`), chains them using
    /// `initial_state_commitment` / `final_state_commitment`, and collects each transaction's
    /// input note nullifiers in execution order. Nullifiers from the same account are in execution
    /// order; ordering across different accounts is arbitrary.
    fn ordered_nullifiers(&self) -> Vec<Nullifier> {
        let mut result = Vec::new();

        for (_, _, txs) in self.iter() {
            for tx in walk_execution_chain(txs) {
                for commitment in tx.transaction_header.input_notes().iter() {
                    result.push(commitment.nullifier());
                }
            }
        }

        result
    }
}

/// Walks a group of transaction records in execution order.
///
/// Same-block transactions for the same account form an execution chain: each tx's
/// `final_state_commitment` is the next tx's `initial_state_commitment`. This finds the chain
/// start and walks forward, yielding each tx in execution order.
fn walk_execution_chain<'a>(
    txs: &'a [&'a RpcTransactionRecord],
) -> impl Iterator<Item = &'a RpcTransactionRecord> + 'a {
    let mut self_loops = Vec::new();
    let mut final_states = BTreeSet::new();
    let mut init_to_tx = BTreeMap::new();

    for tx in txs.iter().copied() {
        let init_state = tx.transaction_header.initial_state_commitment();
        let final_state = tx.transaction_header.final_state_commitment();

        if init_state == final_state {
            self_loops.push(tx);
        } else {
            final_states.insert(final_state);
            init_to_tx.insert(init_state, tx);
        }
    }

    let start = init_to_tx.keys().find(|init| !final_states.contains(init)).copied();

    assert!(start.is_some() || init_to_tx.is_empty(), "cannot walk cyclic execution chain");

    let mut current = start.and_then(|init| init_to_tx.remove(&init));
    let mut self_loops_iter = self_loops.into_iter();

    core::iter::from_fn(move || {
        if let Some(tx) = current {
            current = init_to_tx.remove(&tx.transaction_header.final_state_commitment());
            return Some(tx);
        }
        self_loops_iter.next()
    })
}

#[cfg(all(test, feature = "testing"))]
mod tests;
