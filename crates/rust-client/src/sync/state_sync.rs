use alloc::boxed::Box;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::sync::Arc;
use alloc::vec::Vec;

use async_trait::async_trait;
use miden_protocol::account::{
    Account,
    AccountHeader,
    AccountId,
    AccountStorageHeader,
    StorageMapKey,
    StorageSlotName,
    StorageSlotType,
};
use miden_protocol::block::{BlockHeader, BlockNumber};
use miden_protocol::crypto::merkle::mmr::{MmrDelta, PartialMmr};
use miden_protocol::note::{Note, NoteId, NoteTag, NoteType, Nullifier};
use miden_protocol::{EMPTY_WORD, Word};
use tracing::info;

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
use crate::rpc::domain::account::{AccountDetails, AccountStorageRequirements};
use crate::rpc::domain::note::{CommittedNote, NoteSyncBlock};
use crate::rpc::domain::sync::SyncTarget;
use crate::rpc::domain::transaction::TransactionRecord as RpcTransactionRecord;
use crate::rpc::{AccountStateAt, NodeRpcClient, RpcError};
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
    /// Full note bodies for public notes, keyed by note ID.
    public_notes: BTreeMap<NoteId, Note>,
    /// Transaction records for the synced range, as returned by `sync_transactions`.
    transactions: Vec<RpcTransactionRecord>,
}

// SYNC REQUEST
// ================================================================================================

/// A tracked account passed into a sync, with an optional hint that lets `StateSync` save
/// one RPC roundtrip when the account's storage layout is already known to the caller.
///
/// Hints are purely an optimization: correctness does not depend on them. If `storage_header`
/// has no slots, or if it is stale (a new map slot has appeared on-chain since the hint was
/// produced), `StateSync` transparently falls back to fetching the missing slots and the
/// account is still synced correctly — just at the cost of one extra roundtrip.
#[derive(Debug, Clone)]
pub struct AccountSyncHint {
    /// The account header.
    pub header: AccountHeader,
    /// Local snapshot of the account's storage layout (slot names, types, and current roots
    /// or values). When this carries up-to-date map slot names, `StateSync` can request all
    /// map data in a single `get_account_proof` call. If the on-chain layout has new map
    /// slots, `StateSync` fetches only those missing slots and reuses the already-downloaded
    /// data for the slots covered here.
    pub storage_header: AccountStorageHeader,
}

impl AccountSyncHint {
    /// Creates a hint with no slot information. `StateSync` will discover the slot layout
    /// via an extra RPC call.
    pub fn from_header(header: AccountHeader) -> Self {
        Self {
            header,
            storage_header: AccountStorageHeader::new(Vec::new())
                .expect("an empty storage header is valid"),
        }
    }
}

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
    /// Tracked accounts (with optional storage layout hints) to follow during the sync.
    pub accounts: Vec<AccountSyncHint>,
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
        let account_ids: Vec<AccountId> = accounts.iter().map(|hint| hint.header.id()).collect();

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

        let new_commitments = derive_account_commitments(&sync_data.transactions);
        self.account_state_sync(
            &mut state_sync_update.account_updates,
            &accounts,
            &new_commitments,
            block_num,
        )
        .await?;

        // Apply local changes: update the MMR, screen notes, and apply state transitions.
        self.apply_sync_result(sync_data, &mut state_sync_update, current_partial_mmr)
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

        // Step 2: sync notes and fetch full note bodies for public notes, paginating with the
        // same chain tip so MMR paths are opened at a consistent forest. With no tracked tags
        // there's nothing the node could match, so skip the RPC entirely.
        let (note_blocks, public_notes) = if note_tags.is_empty() {
            (Vec::new(), BTreeMap::new())
        } else {
            self.rpc_api
                .sync_notes_with_details(current_block_num + 1, chain_tip, note_tags.as_ref())
                .await?
        };

        let note_count: usize = note_blocks.iter().map(|b| b.notes.len()).sum();
        info!(
            blocks_with_notes = note_blocks.len(),
            notes = note_count,
            public_notes = public_notes.len(),
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
            public_notes,
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
        state_sync_update: &mut StateSyncUpdate,
        current_partial_mmr: &mut PartialMmr,
    ) -> Result<(), ClientError> {
        let FetchedSyncData {
            mmr_delta,
            chain_tip_header,
            note_blocks,
            public_notes,
            transactions,
        } = sync_data;

        Self::advance_mmr(
            mmr_delta,
            &chain_tip_header,
            current_partial_mmr,
            &mut state_sync_update.partial_blockchain_updates,
        )?;

        self.screen_note_blocks(note_blocks, public_notes, state_sync_update, current_partial_mmr)
            .await?;

        self.apply_transactions_and_nullifiers(
            &chain_tip_header,
            &transactions,
            state_sync_update,
        )?;

        Ok(())
    }

    /// Applies the MMR delta and inserts the chain-tip leaf into the partial blockchain
    /// updates. The delta excludes the chain-tip leaf because of the one-block lag in block
    /// header MMR commitments, so the tip leaf has to be added separately.
    fn advance_mmr(
        mmr_delta: MmrDelta,
        chain_tip_header: &BlockHeader,
        current_partial_mmr: &mut PartialMmr,
        partial_blockchain_updates: &mut PartialBlockchainUpdates,
    ) -> Result<(), ClientError> {
        let mut new_authentication_nodes =
            current_partial_mmr.apply(mmr_delta).map_err(StoreError::MmrError)?;
        partial_blockchain_updates.new_peaks = current_partial_mmr.peaks();
        new_authentication_nodes
            .append(&mut current_partial_mmr.add(chain_tip_header.commitment(), false));

        partial_blockchain_updates.insert(
            chain_tip_header.clone(),
            false,
            new_authentication_nodes,
        );

        Ok(())
    }

    /// Screens each note block for relevance and, for blocks containing client-relevant notes,
    /// tracks them in the partial MMR using the authentication path from the `sync_notes`
    /// response.
    async fn screen_note_blocks(
        &self,
        note_blocks: Vec<NoteSyncBlock>,
        public_notes: BTreeMap<NoteId, Note>,
        state_sync_update: &mut StateSyncUpdate,
        current_partial_mmr: &mut PartialMmr,
    ) -> Result<(), ClientError> {
        let public_note_records = Self::build_public_note_records(public_notes, &note_blocks);

        for block in note_blocks {
            let found_relevant_note = self
                .note_state_sync(
                    &mut state_sync_update.note_updates,
                    block.notes,
                    &block.block_header,
                    &public_note_records,
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
        state_sync_update: &mut StateSyncUpdate,
    ) -> Result<(), ClientError> {
        state_sync_update
            .note_updates
            .extend_nullifiers(compute_ordered_nullifiers(transactions));

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
        accounts: &[AccountSyncHint],
        account_commitment_updates: &[(AccountId, Word)],
        block_from: BlockNumber,
    ) -> Result<(), ClientError> {
        // "Public" here includes both Public and Network accounts, since both have
        // their state stored on-chain and follow the same sync path.
        let (public_accounts, private_accounts): (Vec<_>, Vec<_>) =
            accounts.iter().partition(|hint| !hint.header.id().is_private());

        self.sync_public_accounts(
            account_updates,
            account_commitment_updates,
            &public_accounts,
            block_from,
        )
        .await?;

        let mismatched_private_accounts = account_commitment_updates
            .iter()
            .filter(|(account_id, digest)| {
                private_accounts.iter().any(|hint| {
                    hint.header.id() == *account_id && &hint.header.to_commitment() != digest
                })
            })
            .copied()
            .collect::<Vec<_>>();

        account_updates.extend(AccountUpdates::new(Vec::new(), mismatched_private_accounts));

        Ok(())
    }

    /// Queries the node for updated public accounts and populates `account_updates`.
    ///
    /// For each public account whose commitment changed, an updated snapshot is fetched via
    /// `get_account_proof`. Callers may supply [`AccountSyncHint::map_slot_names`] to request
    /// map storage data up-front and avoid a roundtrip; otherwise a second call is issued when
    /// the account turns out to have map slots.
    ///
    /// Accounts whose vault or maps are too large to fit in a single response fall back to the
    /// incremental [`PublicAccountUpdate::Delta`] path, which fetches vault and storage map
    /// updates over the synced block range.
    async fn sync_public_accounts(
        &self,
        account_updates: &mut AccountUpdates,
        commitment_updates: &[(AccountId, Word)],
        current_public_accounts: &[&AccountSyncHint],
        block_from: BlockNumber,
    ) -> Result<(), ClientError> {
        for (id, commitment) in commitment_updates {
            let Some(local_hint) = current_public_accounts
                .iter()
                .find(|hint| *id == hint.header.id() && *commitment != hint.header.to_commitment())
            else {
                continue;
            };

            let public_update = self.sync_public_account(local_hint, block_from).await?;
            account_updates.extend(AccountUpdates::new(vec![public_update], Vec::new()));
        }

        Ok(())
    }

    /// Fetches an updated snapshot for a single public account.
    ///
    /// Must only be called when the local commitment for `local_hint` is known to differ from the
    /// network's, which guarantees the node returns updated details with a newer nonce.
    ///
    /// # Panics
    ///
    /// Panics if the node response omits account details, since that would mean the account is
    /// not public.
    async fn sync_public_account(
        &self,
        local_hint: &AccountSyncHint,
        block_from: BlockNumber,
    ) -> Result<PublicAccountUpdate, ClientError> {
        let account_id = local_hint.header.id();

        // Map slot names already known locally via the hint.
        let hinted_map_slots: Vec<StorageSlotName> = local_hint
            .storage_header
            .slots()
            .filter(|slot| slot.slot_type() == StorageSlotType::Map)
            .map(|slot| slot.name().clone())
            .collect();

        // Request all map data we know about up-front so the response is self-sufficient
        // when the on-chain layout hasn't grown since the hint was produced.
        let initial_requirements = AccountStorageRequirements::new(
            hinted_map_slots
                .iter()
                .map(|n| (n.clone(), core::iter::empty::<&StorageMapKey>())),
        );

        let (proof_block_num, proof) = self
            .rpc_api
            .get_account_proof(
                account_id,
                initial_requirements,
                AccountStateAt::ChainTip,
                None,
                Some(EMPTY_WORD),
            )
            .await
            .map_err(ClientError::RpcError)?;

        let details = proof.into_details().expect("node returned no details for a public account");

        let vault_oversized = details.vault_details.too_many_assets;
        let any_map_oversized =
            details.storage_details.map_details.iter().any(|m| m.too_many_entries);

        // Map slot names actually present on the account, taken from the response header.
        let response_map_slots: Vec<StorageSlotName> = details
            .storage_details
            .header
            .slots()
            .filter(|slot| slot.slot_type() == StorageSlotType::Map)
            .map(|slot| slot.name().clone())
            .collect();

        // Slots present on-chain that weren't covered by the hint — we still need to fetch
        // their entries.
        let missing_map_slots: Vec<StorageSlotName> = response_map_slots
            .iter()
            .filter(|name| !hinted_map_slots.iter().any(|h| h == *name))
            .cloned()
            .collect();

        let public_update = if vault_oversized || any_map_oversized {
            // Some part of the account is oversized — use incremental endpoints.
            self.build_delta_update(account_id, &details, block_from, proof_block_num)
                .await?
        } else if missing_map_slots.is_empty() {
            // The hint covered every map slot the account actually has, so the initial
            // response already carries all the map data we need.
            let account = Account::try_from(&details).map_err(ClientError::RpcError)?;
            PublicAccountUpdate::Full(account)
        } else {
            // Hint is incomplete (new map slots appeared since the last sync). Fetch only the
            // missing slots and merge their entries into the response we already have.
            self.fetch_missing_map_data(
                account_id,
                details,
                &missing_map_slots,
                block_from,
                proof_block_num,
            )
            .await?
        };

        Ok(public_update)
    }

    /// Fetches map data for the slots not covered by the hint and merges them into the entries
    /// already received in the initial response. Falls back to the
    /// [`PublicAccountUpdate::Delta`] path if the follow-up response reveals oversized maps.
    async fn fetch_missing_map_data(
        &self,
        account_id: AccountId,
        mut initial_details: AccountDetails,
        missing_map_slots: &[StorageSlotName],
        block_from: BlockNumber,
        block_to: BlockNumber,
    ) -> Result<PublicAccountUpdate, ClientError> {
        let storage_requirements = AccountStorageRequirements::new(
            missing_map_slots
                .iter()
                .map(|n| (n.clone(), core::iter::empty::<&StorageMapKey>())),
        );

        let (_, follow_up_proof) = self
            .rpc_api
            .get_account_proof(
                account_id,
                storage_requirements,
                AccountStateAt::Block(block_to),
                Some(initial_details.code.clone()),
                Some(EMPTY_WORD),
            )
            .await
            .map_err(ClientError::RpcError)?;

        let Some(follow_up_details) = follow_up_proof.into_details() else {
            return Err(ClientError::RpcError(RpcError::ExpectedDataMissing(
                "follow-up get_account_proof returned no details for a public account".into(),
            )));
        };

        let any_oversized =
            follow_up_details.storage_details.map_details.iter().any(|m| m.too_many_entries);
        if any_oversized {
            return self
                .build_delta_update(account_id, &initial_details, block_from, block_to)
                .await;
        }

        // Merge the follow-up map entries into the initial response. The initial response
        // already carries the storage header (with every slot's root) and the entries for the
        // hinted slots; we only need to graft on the entries for the slots we just fetched.
        initial_details
            .storage_details
            .map_details
            .extend(follow_up_details.storage_details.map_details);

        let account = Account::try_from(&initial_details).map_err(ClientError::RpcError)?;
        Ok(PublicAccountUpdate::Full(account))
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
            .sync_storage_maps(block_from + 1, Some(block_to), account_id)
            .await
            .map_err(ClientError::RpcError)?;
        let vault_info = self
            .rpc_api
            .sync_account_vault(block_from + 1, Some(block_to), account_id)
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
    /// iteration so the node is only queried once per batch.
    async fn note_state_sync(
        &self,
        note_updates: &mut NoteUpdateTracker,
        note_inclusions: BTreeMap<NoteId, CommittedNote>,
        block_header: &BlockHeader,
        public_notes: &BTreeMap<NoteId, InputNoteRecord>,
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
            .sync_nullifiers(
                &nullifiers_tags,
                current_block_num + 1,
                Some(state_sync_update.block_num),
            )
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

    /// Pairs each public note body with the matching inclusion proof from `note_blocks`. Notes
    /// without a matching inclusion proof are dropped.
    fn build_public_note_records(
        public_notes: BTreeMap<NoteId, Note>,
        note_blocks: &[NoteSyncBlock],
    ) -> BTreeMap<NoteId, InputNoteRecord> {
        let mut records = BTreeMap::new();
        for (note_id, note) in public_notes {
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
                records.insert(record.id(), record);
            }
        }
        records
    }
}

/// Groups transaction records by `(account_id, block_num)`.
fn group_txs_by_account_block(
    transaction_records: &[RpcTransactionRecord],
) -> BTreeMap<(AccountId, BlockNumber), Vec<&RpcTransactionRecord>> {
    let mut groups: BTreeMap<(AccountId, BlockNumber), Vec<&RpcTransactionRecord>> =
        BTreeMap::new();
    for record in transaction_records {
        let account_id = record.transaction_header.account_id();
        groups.entry((account_id, record.block_num)).or_default().push(record);
    }
    groups
}

/// Walks a group of transaction records in execution order.
///
/// Same-block transactions for the same account form an execution chain: each tx's
/// `final_state_commitment` is the next tx's `initial_state_commitment`. This finds the chain
/// start and walks forward, yielding each tx in execution order.
fn walk_execution_chain<'a>(
    txs: &'a [&'a RpcTransactionRecord],
) -> impl Iterator<Item = &'a RpcTransactionRecord> + 'a {
    let (self_loops, chained): (Vec<&RpcTransactionRecord>, Vec<&RpcTransactionRecord>) =
        txs.iter().copied().partition(|tx| {
            tx.transaction_header.initial_state_commitment()
                == tx.transaction_header.final_state_commitment()
        });

    let final_states: BTreeSet<Word> = chained
        .iter()
        .map(|tx| tx.transaction_header.final_state_commitment())
        .collect();

    let mut init_to_tx: BTreeMap<Word, &RpcTransactionRecord> = chained
        .iter()
        .map(|tx| (tx.transaction_header.initial_state_commitment(), *tx))
        .collect();

    let start = chained
        .iter()
        .find(|tx| !final_states.contains(&tx.transaction_header.initial_state_commitment()))
        .copied();

    assert!(start.is_some() || chained.is_empty(), "cannot walk cyclic execution chain");

    let mut current =
        start.and_then(|tx| init_to_tx.remove(&tx.transaction_header.initial_state_commitment()));
    let mut self_loops_iter = self_loops.into_iter();

    core::iter::from_fn(move || {
        if let Some(tx) = current {
            current = init_to_tx.remove(&tx.transaction_header.final_state_commitment());
            return Some(tx);
        }
        self_loops_iter.next()
    })
}

/// For each unique account, returns the `final_state_commitment` from the final transaction with
/// the highest `block_num`.
fn derive_account_commitments(
    transaction_records: &[RpcTransactionRecord],
) -> Vec<(AccountId, Word)> {
    let mut latest_by_account: BTreeMap<AccountId, (BlockNumber, Word)> = BTreeMap::new();

    for ((account_id, block_num), txs) in &group_txs_by_account_block(transaction_records) {
        let terminal_state = walk_execution_chain(txs)
            .last()
            .expect("account must have a final state")
            .transaction_header
            .final_state_commitment();

        latest_by_account
            .entry(*account_id)
            .and_modify(|(existing_block, existing_state)| {
                if *block_num > *existing_block {
                    *existing_block = *block_num;
                    *existing_state = terminal_state;
                }
            })
            .or_insert((*block_num, terminal_state));
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
fn compute_ordered_nullifiers(transaction_records: &[RpcTransactionRecord]) -> Vec<Nullifier> {
    let mut result = Vec::new();

    for txs in group_txs_by_account_block(transaction_records).values() {
        for tx in walk_execution_chain(txs) {
            for commitment in tx.transaction_header.input_notes().iter() {
                result.push(commitment.nullifier());
            }
        }
    }

    result
}

#[cfg(all(test, feature = "testing"))]
mod tests {
    use alloc::collections::BTreeSet;
    use alloc::sync::Arc;

    use async_trait::async_trait;
    use miden_protocol::account::Account;
    use miden_protocol::assembly::DefaultSourceManager;
    use miden_protocol::asset::{Asset, FungibleAsset};
    use miden_protocol::block::BlockNumber;
    use miden_protocol::crypto::merkle::mmr::{Forest, InOrderIndex, PartialMmr};
    use miden_protocol::note::{
        Note,
        NoteAssets,
        NoteAttachment,
        NoteAttachments,
        NoteHeader,
        NoteMetadata,
        NoteRecipient,
        NoteStorage,
        NoteTag,
        NoteType,
        PartialNoteMetadata,
    };
    use miden_protocol::testing::account_id::{
        ACCOUNT_ID_PRIVATE_FUNGIBLE_FAUCET,
        ACCOUNT_ID_PUBLIC_FUNGIBLE_FAUCET,
        ACCOUNT_ID_REGULAR_NETWORK_ACCOUNT_IMMUTABLE_CODE,
        ACCOUNT_ID_REGULAR_PRIVATE_ACCOUNT_UPDATABLE_CODE,
        ACCOUNT_ID_SENDER,
    };
    use miden_protocol::transaction::{InputNotes, TransactionHeader};
    use miden_protocol::{Felt, Word, ZERO};
    use miden_standards::code_builder::CodeBuilder;
    use miden_standards::note::{NetworkAccountTarget, NoteExecutionHint};
    use miden_testing::{MockChainBuilder, TxContextInput};

    use super::*;
    use crate::rpc::domain::transaction::ACCOUNT_ID_NATIVE_ASSET_FAUCET;
    use crate::store::{OutputNoteRecord, OutputNoteState};
    use crate::test_utils::mock::MockRpcApi;

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

    fn word(n: u64) -> miden_protocol::Word {
        [Felt::new(n), ZERO, ZERO, ZERO].into()
    }

    // COMPUTE NULLIFIER TX ORDER TESTS
    // --------------------------------------------------------------------------------------------

    mod compute_nullifiers_tests {
        use alloc::vec;

        use miden_protocol::asset::FungibleAsset;
        use miden_protocol::block::BlockNumber;
        use miden_protocol::note::Nullifier;
        use miden_protocol::transaction::{InputNoteCommitment, InputNotes, TransactionHeader};

        use super::word;
        use crate::rpc::domain::transaction::{
            ACCOUNT_ID_NATIVE_ASSET_FAUCET,
            TransactionRecord as RpcTransactionRecord,
        };

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
                output_notes: vec![],
                erased_output_notes: vec![],
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
                output_notes: vec![],
                erased_output_notes: vec![],
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

    // DERIVE ACCOUNT COMMITMENTS TESTS
    // --------------------------------------------------------------------------------------------

    /// `derive_account_commitments` must walk the execution chain to get the final commitment when
    /// several transactions for the same account land in the same block.
    ///
    /// Test scenario:
    /// - Account A, block 5: chain 1 - 2 - 3 (older group; must be dominated by block 6).
    /// - Account A, block 6: chain 3 - 4 - 5 (final state = 5).
    /// - Account B, block 6: single tx 10 - 20 (final state = 20).
    #[test]
    fn derive_account_commitments_walks_chains_per_account() {
        let fee =
            FungibleAsset::new(ACCOUNT_ID_NATIVE_ASSET_FAUCET.try_into().expect("valid"), 0u64)
                .unwrap();
        let make_tx = |account: AccountId, init_state: u64, final_state: u64, block_num: u32| {
            RpcTransactionRecord {
                block_num: BlockNumber::from(block_num),
                transaction_header: TransactionHeader::new(
                    account,
                    word(init_state),
                    word(final_state),
                    InputNotes::new_unchecked(vec![]),
                    vec![],
                    fee,
                ),
                output_notes: vec![],
                erased_output_notes: vec![],
            }
        };

        let account_a: AccountId =
            ACCOUNT_ID_REGULAR_PRIVATE_ACCOUNT_UPDATABLE_CODE.try_into().unwrap();
        let account_b: AccountId = ACCOUNT_ID_PUBLIC_FUNGIBLE_FAUCET.try_into().unwrap();

        let tx_a_b5_1 = make_tx(account_a, 1, 2, 5);
        let tx_a_b5_2 = make_tx(account_a, 2, 3, 5);
        let tx_a_b6_1 = make_tx(account_a, 3, 4, 6);
        let tx_a_b6_2 = make_tx(account_a, 4, 5, 6);
        let tx_b_b6 = make_tx(account_b, 10, 20, 6);

        // Insert transactions not ordered by execution order.
        let result = super::derive_account_commitments(&[
            tx_a_b6_1, tx_b_b6, tx_a_b5_2, tx_a_b6_2, tx_a_b5_1,
        ]);

        assert_eq!(result.len(), 2, "one entry per account");
        assert!(
            result.contains(&(account_a, word(5))),
            "account A: must walk block 6's chain, not return block 5 or an intermediate",
        );
        assert!(
            result.contains(&(account_b, word(20))),
            "account B: must be resolved independently of account A",
        );
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

    /// Builds a `MockChain` where 3 notes are consumed by chained transactions in the same block.
    ///
    /// Returns the chain, the account, and the 3 notes (in consumption order).
    async fn build_chain_with_chained_consume_txs() -> (miden_testing::MockChain, Account, [Note; 3])
    {
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

        let account_id = account.id();
        let sync_input = StateSyncInput {
            accounts: vec![AccountSyncHint::from_header(AccountHeader::from(account))],
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

        // Since there are no uncommitted_transactions, these notes were consumed by a tracked
        // account via external transactions. Verify that consumer_account is populated.
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
                    call.::miden::standards::faucets::fungible::mint_and_send
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
        let note_blocks = mock_rpc
            .sync_notes(BlockNumber::from(0u32), chain_tip, &note_tags)
            .await
            .unwrap();
        assert!(
            note_blocks.len() >= 2,
            "expected notes in multiple blocks, got {}",
            note_blocks.len()
        );

        // Collect the block numbers that have notes.
        let note_block_nums: BTreeSet<BlockNumber> =
            note_blocks.iter().map(|b| b.block_header.block_num()).collect();

        // Test that fetch_sync_data returns note blocks with valid MMR paths that
        // can be used to track blocks in the partial MMR.
        let state_sync = StateSync::new(Arc::new(mock_rpc.clone()), Arc::new(MockScreener), None);

        let genesis_peaks = mock_rpc.get_mmr().peaks_at(Forest::new(1)).unwrap();
        let mut partial_mmr = PartialMmr::from_peaks(genesis_peaks);

        let sync_data = state_sync
            .fetch_sync_data(BlockNumber::GENESIS, &[], &Arc::new(note_tags.clone()))
            .await
            .unwrap()
            .expect("should have progressed past genesis");

        // Should have advanced to the chain tip.
        assert_eq!(sync_data.chain_tip_header.block_num(), chain_tip);
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

    #[tokio::test]
    async fn sync_notes_with_details_fetches_inclusive_upper_bound_page() {
        let (chain, note_tags) = build_chain_with_mint_notes(10).await;
        let mock_rpc = MockRpcApi::new(chain);

        let (blocks, _public_notes) = mock_rpc
            .sync_notes_with_details(4_u32.into(), 10_u32.into(), &note_tags)
            .await
            .expect("sync notes should succeed");

        assert_eq!(blocks.last().unwrap().block_header.block_num(), BlockNumber::from(10u32));
        assert!(
            blocks
                .iter()
                .any(|block| block.block_header.block_num() == BlockNumber::from(9u32))
        );
    }

    /// Tests that erased notes are marked as consumed when a committed transaction
    /// reports output notes that were erased by same-batch note erasure.
    ///
    /// This simulates same-batch note erasure: the transaction was committed, its header
    /// says it produced a note, but the note was erased and doesn't exist on the node.
    #[tokio::test]
    async fn erased_notes_are_marked_as_consumed() {
        // Create a public output note. It won't be in the mock chain (simulating erasure).
        let sender_id: AccountId = ACCOUNT_ID_SENDER.try_into().unwrap();
        let partial_metadata = PartialNoteMetadata::new(sender_id, NoteType::Public);
        let metadata = NoteMetadata::new(partial_metadata, &NoteAttachments::empty());
        let script = CodeBuilder::new()
            .compile_note_script("@note_script\npub proc main\n    nop\nend")
            .unwrap();
        let recipient = NoteRecipient::new(
            Word::from([Felt::new(1), Felt::new(2), Felt::new(3), Felt::new(4)]),
            script,
            NoteStorage::new(vec![]).unwrap(),
        );
        let output_note = OutputNoteRecord::new(
            recipient.digest(),
            NoteAssets::new(vec![]).unwrap(),
            metadata,
            OutputNoteState::ExpectedFull { recipient },
            BlockNumber::from(1u32),
        );
        let note_id = output_note.id();
        let note_header = NoteHeader::new(note_id, metadata);

        // Build a NoteUpdateTracker with the output note.
        let mut note_updates = NoteUpdateTracker::new(vec![], vec![output_note]);

        // Mark the note as erased (created and consumed in the same batch).
        let block_num = BlockNumber::from(3u32);
        note_updates
            .mark_erased_note_as_consumed(&note_header, block_num)
            .expect("marking erased note should succeed");

        let updated = note_updates
            .updated_output_notes()
            .find(|n| n.id() == note_id)
            .expect("output note should be in the update");

        assert!(
            updated.inner().is_consumed(),
            "output note should be consumed after erasure detection, but state is: {}",
            updated.inner().state()
        );
    }

    /// Exercises the full sync flow for an erased output note that targets a network account.
    ///
    /// Same-batch erasure scenario: a sender's transaction creates an output note targeting a
    /// network account that consumes it in the same batch, so the note never appears in the
    /// block body and the mock RPC surfaces it as erased in the transaction sync response.
    ///
    /// The consumer account is not derivable from the erased note's [`NoteHeader`] (the network
    /// target lives in the attachment content, which the erased-note stream does not deliver), so
    /// the output note is marked consumed but no input note record is attributed to the network
    /// account.
    #[tokio::test]
    #[allow(clippy::too_many_lines)]
    async fn erased_notes_are_marked_as_consumed_by_network_account() {
        // Build a chain with a sender that executes one tx so `sync_transactions` returns
        // a record. The mock attaches the registered erased note header to that record.
        let mut builder = MockChainBuilder::new();
        let p2id_sender: AccountId = ACCOUNT_ID_SENDER.try_into().unwrap();
        let faucet_id: AccountId = ACCOUNT_ID_PRIVATE_FUNGIBLE_FAUCET.try_into().unwrap();
        let sender_account =
            builder.add_existing_mock_account(miden_testing::Auth::IncrNonce).unwrap();
        let sender_id = sender_account.id();

        let asset = Asset::Fungible(FungibleAsset::new(faucet_id, 100u64).unwrap());
        let note = builder
            .add_p2id_note(p2id_sender, sender_id, &[asset], NoteType::Public)
            .unwrap();

        let mut chain = builder.build().unwrap();
        chain.prove_next_block().unwrap();

        let tx = Box::pin(
            chain
                .build_tx_context(
                    TxContextInput::Account(sender_account.clone()),
                    &[],
                    core::slice::from_ref(&note),
                )
                .unwrap()
                .build()
                .unwrap()
                .execute(),
        )
        .await
        .unwrap();
        chain.add_pending_executed_transaction(&tx).unwrap();
        chain.prove_next_block().unwrap();

        // Construct the erased note that targets the network account.
        let network_account_id: AccountId =
            ACCOUNT_ID_REGULAR_NETWORK_ACCOUNT_IMMUTABLE_CODE.try_into().unwrap();
        let target =
            NetworkAccountTarget::new(network_account_id, NoteExecutionHint::Always).unwrap();
        let attachment: NoteAttachment = target.into();
        let attachments = NoteAttachments::new(vec![attachment]).unwrap();
        let partial_metadata = PartialNoteMetadata::new(sender_id, NoteType::Public);
        let metadata = NoteMetadata::new(partial_metadata, &attachments);
        let script = CodeBuilder::new()
            .compile_note_script("@note_script\npub proc main\n    nop\nend")
            .unwrap();
        let recipient = NoteRecipient::new(
            Word::from([Felt::new(7), Felt::new(8), Felt::new(9), Felt::new(10)]),
            script,
            NoteStorage::new(vec![]).unwrap(),
        );
        let recipient_digest = recipient.digest();
        let assets = NoteAssets::new(vec![]).unwrap();

        // Output note record tracked by the sender prior to sync.
        let output_note = OutputNoteRecord::new(
            recipient_digest,
            assets,
            metadata,
            OutputNoteState::ExpectedFull { recipient },
            BlockNumber::from(1u32),
        );
        let erased_note_id = output_note.id();
        let erased_note_header = NoteHeader::new(erased_note_id, metadata);

        let mock_rpc = MockRpcApi::new(chain);
        mock_rpc.mark_note_as_erased(erased_note_header);

        // Track both the sender (so its tx is returned) and the network account.
        let network_header = AccountHeader::new(
            network_account_id,
            Felt::new(0),
            EMPTY_WORD,
            EMPTY_WORD,
            EMPTY_WORD,
        );

        let state_sync = StateSync::new(Arc::new(mock_rpc.clone()), Arc::new(MockScreener), None);

        let genesis_peaks = mock_rpc.get_mmr().peaks_at(Forest::new(1)).unwrap();
        let mut partial_mmr = PartialMmr::from_peaks(genesis_peaks);

        let sync_input = StateSyncInput {
            accounts: vec![
                AccountSyncHint::from_header(AccountHeader::from(sender_account)),
                AccountSyncHint::from_header(network_header),
            ],
            note_tags: BTreeSet::new(),
            input_notes: vec![],
            output_notes: vec![output_note],
            uncommitted_transactions: vec![],
        };

        let update = state_sync.sync_state(&mut partial_mmr, sync_input).await.unwrap();

        // The output note record should transition to consumed.
        let updated_output = update
            .note_updates
            .updated_output_notes()
            .find(|n| n.id() == erased_note_id)
            .expect("output note should be in the update");
        assert!(
            updated_output.inner().is_consumed(),
            "output note should be consumed, got: {}",
            updated_output.inner().state()
        );

        // The consumer is not derivable from the erased note header, so no input note record is
        // attributed to the network account.
        assert!(
            update.note_updates.updated_input_notes().all(|n| n.id() != erased_note_id),
            "no input note should be attributed to the network account for an erased note",
        );
    }
}
