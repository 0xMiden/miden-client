use alloc::boxed::Box;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::sync::Arc;
use alloc::vec::Vec;

use async_trait::async_trait;
use miden_protocol::account::{
    Account,
    AccountCode,
    AccountDelta,
    AccountHeader,
    AccountId,
    AccountStorage,
    AccountStorageDelta,
    AccountVaultDelta,
    StorageMapKey,
    StorageSlot,
    StorageSlotContent,
    StorageSlotName,
    StorageSlotType,
};
use miden_protocol::asset::{Asset, AssetVault, AssetVaultKey};
use miden_protocol::block::{BlockHeader, BlockNumber};
use miden_protocol::crypto::merkle::mmr::{MmrDelta, PartialMmr};
use miden_protocol::note::{Note, NoteId, NoteTag, NoteType, Nullifier};
use miden_protocol::transaction::InputNoteCommitment;
use miden_protocol::{Felt, Word};
use tracing::info;

use super::state_sync_update::TransactionUpdateTracker;
use super::{AccountUpdates, PublicAccountUpdate, StateSyncUpdate};
use crate::ClientError;
use crate::note::NoteUpdateTracker;
use crate::rpc::domain::account::{
    AccountDetails,
    AccountStorageMapDetails,
    AccountStorageRequirements,
    FetchedAccount,
};
use crate::rpc::domain::note::{CommittedNote, NoteSyncBlock};
use crate::rpc::domain::storage_map::StorageMapUpdate;
use crate::rpc::domain::transaction::{
    TransactionInclusion,
    TransactionRecord as RpcTransactionRecord,
};
use crate::rpc::{AccountStateAt, NodeRpcClient, RpcError};
use crate::store::{AccountStorageFilter, InputNoteRecord, OutputNoteRecord, Store, StoreError};
use crate::transaction::TransactionRecord;

// STATE UPDATE DATA
// ================================================================================================

/// Raw data fetched from the node needed to sync the client to the chain tip.
///
/// Aggregates the responses of `sync_chain_mmr`, `sync_notes`, `get_notes_by_id`, and
/// `sync_transactions`. This may contain more data than a particular client needs to store — it is
/// filtered and transformed into a [`StateSyncUpdate`] before being applied.
struct RawStateSyncData {
    /// MMR delta covering the full range from `current_block` to `chain_tip`.
    mmr_delta: MmrDelta,
    /// Chain tip block header.
    chain_tip_header: BlockHeader,
    /// Blocks with matching notes that the client is interested in.
    note_blocks: Vec<NoteSyncBlock>,
    /// Full note bodies for public notes, keyed by note ID.
    public_notes: BTreeMap<NoteId, Note>,
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
    /// The client's store, used to fetch account storage and vault data on demand during
    /// delta-based sync of public accounts. When `None`, oversized public accounts fall back
    /// to `get_account_details` (full sync from block 0).
    store: Option<Arc<dyn Store>>,
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
    /// * `store` - Optional store for on-demand account data access during delta sync.
    /// * `note_screener` - The note screener used to check the relevance of notes.
    /// * `tx_discard_delta` - Number of blocks after which pending transactions are discarded.
    pub fn new(
        rpc_api: Arc<dyn NodeRpcClient>,
        store: Option<Arc<dyn Store>>,
        note_screener: Arc<dyn OnNoteReceived>,
        tx_discard_delta: Option<u32>,
    ) -> Self {
        Self {
            rpc_api,
            store,
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
        let Some(mut sync_data) = self
            .fetch_sync_data(state_sync_update.block_num, &account_ids, &note_tags)
            .await?
        else {
            // No progress — already at the tip.
            return Ok(state_sync_update);
        };

        state_sync_update.block_num = sync_data.chain_tip_header.block_num();

        // Build input note records for public notes from the fetched note bodies and the
        // inclusion proofs already present in the note blocks.
        let mut public_note_records: BTreeMap<NoteId, InputNoteRecord> = BTreeMap::new();
        for (note_id, note) in core::mem::take(&mut sync_data.public_notes) {
            let inclusion_proof = sync_data
                .note_blocks
                .iter()
                .find_map(|b| b.notes.get(&note_id))
                .map(|committed| committed.inclusion_proof().clone());

            if let Some(inclusion_proof) = inclusion_proof {
                let state = crate::store::input_note_states::UnverifiedNoteState {
                    metadata: note.metadata().clone(),
                    inclusion_proof,
                }
                .into();
                let record = InputNoteRecord::new(note.into(), None, state);
                public_note_records.insert(record.id(), record);
            }
        }

        self.account_state_sync(
            &mut state_sync_update.account_updates,
            &accounts,
            &sync_data.account_commitment_updates,
            block_num,
        )
        .await?;

        // Apply local changes: update the MMR, screen notes, and apply state transitions.
        self.apply_sync_result(
            sync_data,
            &public_note_records,
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
    ) -> Result<Option<RawStateSyncData>, ClientError> {
        // Step 1: Fetch the MMR delta and chain tip header.
        let chain_mmr_info = self.rpc_api.sync_chain_mmr(current_block_num, None).await?;
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

        // Step 2: Paginate sync_notes using the same chain tip so MMR paths are opened at
        // a consistent forest.
        let sync_notes_result = self
            .rpc_api
            .sync_notes_with_details(current_block_num, Some(chain_tip), note_tags.as_ref())
            .await?;

        let note_count: usize = sync_notes_result.blocks.iter().map(|b| b.notes.len()).sum();
        info!(
            blocks_with_notes = sync_notes_result.blocks.len(),
            notes = note_count,
            public_notes = sync_notes_result.public_notes.len(),
            "Fetched note sync data.",
        );

        // Step 3: Gather transactions for tracked accounts over the full range.
        let (account_commitment_updates, transactions, nullifiers) =
            self.fetch_transaction_data(current_block_num, chain_tip, account_ids).await?;

        Ok(Some(RawStateSyncData {
            mmr_delta: chain_mmr_info.mmr_delta,
            chain_tip_header: chain_mmr_info.block_header,
            note_blocks: sync_notes_result.blocks,
            public_notes: sync_notes_result.public_notes,
            account_commitment_updates,
            transactions,
            nullifiers,
        }))
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

        let transaction_records = tx_info.transaction_records;

        let account_updates = derive_account_commitment_updates(&transaction_records);
        let nullifiers = compute_ordered_nullifiers(&transaction_records);

        let tx_inclusions = transaction_records
            .into_iter()
            .map(|r| {
                let nullifiers = r
                    .transaction_header
                    .input_notes()
                    .iter()
                    .map(InputNoteCommitment::nullifier)
                    .collect();
                TransactionInclusion {
                    transaction_id: r.transaction_header.id(),
                    block_num: r.block_num,
                    account_id: r.transaction_header.account_id(),
                    initial_state_commitment: r.transaction_header.initial_state_commitment(),
                    nullifiers,
                    output_notes: r.output_notes,
                }
            })
            .collect();

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
        sync_data: RawStateSyncData,
        public_note_records: &BTreeMap<NoteId, InputNoteRecord>,
        state_sync_update: &mut StateSyncUpdate,
        current_partial_mmr: &mut PartialMmr,
    ) -> Result<(), ClientError> {
        let RawStateSyncData {
            mmr_delta,
            chain_tip_header,
            note_blocks,
            nullifiers,
            transactions,
            ..
        } = sync_data;

        // Advance the partial MMR: apply delta (up to chain_tip - 1), capture peaks for
        // storage, then add the chain tip leaf (which the delta excludes due to the
        // one-block lag in block header MMR commitments).
        let mut new_authentication_nodes =
            current_partial_mmr.apply(mmr_delta).map_err(StoreError::MmrError)?;
        let new_peaks = current_partial_mmr.peaks();
        new_authentication_nodes
            .append(&mut current_partial_mmr.add(chain_tip_header.commitment(), false));

        state_sync_update.block_updates.insert(
            chain_tip_header.clone(),
            false,
            new_peaks,
            new_authentication_nodes,
        );

        // Screen each note block and track relevant ones in the partial MMR using the
        // authentication path from the sync_notes response.
        for block in note_blocks {
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
        state_sync_update.note_updates.extend_nullifiers(nullifiers);
        self.transaction_state_sync(
            &mut state_sync_update.transaction_updates,
            &chain_tip_header,
            &transactions,
        );

        // Transition tracked output notes to Committed using inclusion proofs from the
        // transaction sync response. This covers output notes regardless of whether their
        // tags were tracked in the note sync.
        for transaction in &transactions {
            state_sync_update
                .note_updates
                .apply_output_note_inclusion_proofs(&transaction.output_notes)?;
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
        block_num: BlockNumber,
    ) -> Result<(), ClientError> {
        let (public_accounts, private_accounts): (Vec<_>, Vec<_>) =
            accounts.iter().partition(|a| !a.id().is_private());

        self.sync_public_accounts(
            account_updates,
            account_commitment_updates,
            &public_accounts,
            block_num,
        )
        .await?;

        let mismatched_private_accounts = account_commitment_updates
            .iter()
            .filter(|(account_id, digest)| {
                private_accounts
                    .iter()
                    .any(|a| a.id() == *account_id && &a.to_commitment() != digest)
            })
            .copied()
            .collect::<Vec<_>>();

        account_updates.extend(AccountUpdates::new(Vec::new(), mismatched_private_accounts));

        Ok(())
    }

    /// Queries the node for updated public accounts and populates `account_updates`.
    ///
    /// When a store is available, storage and vault data are fetched on demand to build
    /// deltas for oversized accounts. Without a store, oversized accounts fall back to
    /// `get_account_details` (full sync from block 0).
    async fn sync_public_accounts(
        &self,
        account_updates: &mut AccountUpdates,
        commitment_updates: &[(AccountId, Word)],
        current_public_accounts: &[&AccountHeader],
        block_num: BlockNumber,
    ) -> Result<(), ClientError> {
        for (id, commitment) in commitment_updates {
            let Some(local_header) = current_public_accounts
                .iter()
                .find(|acc| *id == acc.id() && *commitment != acc.to_commitment())
            else {
                continue;
            };

            let account_id = local_header.id();

            // Build storage requirements and known code from store (if available) to
            // request all entries for every map slot and avoid re-downloading code.
            let (storage_requirements, known_code) =
                self.fetch_local_account_hints(account_id).await;

            let (_proof_block_num, proof) = self
                .rpc_api
                .get_account_proof(
                    account_id,
                    storage_requirements,
                    AccountStateAt::ChainTip,
                    known_code,
                    None,
                )
                .await
                .map_err(ClientError::RpcError)?;

            let Some(details) = proof.into_parts().1 else {
                // Private account returned — should not happen for public accounts.
                continue;
            };

            // Skip if the remote nonce is not newer than what we already have.
            if details.header.nonce().as_canonical_u64() <= local_header.nonce().as_canonical_u64()
            {
                continue;
            }

            let has_oversized_data = details.vault_details.too_many_assets
                || details.storage_details.map_details.iter().any(|m| m.too_many_entries);

            if has_oversized_data {
                if self.store.is_some() {
                    // Delta path: build an AccountDelta from incremental updates,
                    // fetching storage slots and vault from the store on demand.
                    let delta = self.build_account_delta(&details, local_header, block_num).await?;
                    account_updates.extend(AccountUpdates::new(
                        vec![PublicAccountUpdate::Delta {
                            new_header: details.header.clone(),
                            delta,
                        }],
                        Vec::new(),
                    ));
                } else {
                    // No store available — fall back to get_account_details which
                    // handles oversized data internally (syncing from block 0).
                    let response = self
                        .rpc_api
                        .get_account_details(account_id)
                        .await
                        .map_err(ClientError::RpcError)?;

                    match response {
                        FetchedAccount::Public(account, _) => {
                            account_updates.extend(AccountUpdates::new(
                                vec![PublicAccountUpdate::Full(*account)],
                                Vec::new(),
                            ));
                        },
                        FetchedAccount::Private(..) => {},
                    }
                }
            } else {
                // Small account: build directly from the response details.
                let account =
                    Self::build_account_from_details(&details).map_err(ClientError::RpcError)?;
                account_updates.extend(AccountUpdates::new(
                    vec![PublicAccountUpdate::Full(account)],
                    Vec::new(),
                ));
            }
        }

        Ok(())
    }

    /// Fetches storage requirements and known code from the store for a given account.
    ///
    /// Returns defaults when no store is available.
    async fn fetch_local_account_hints(
        &self,
        account_id: AccountId,
    ) -> (AccountStorageRequirements, Option<AccountCode>) {
        let Some(store) = &self.store else {
            return (AccountStorageRequirements::default(), None);
        };

        let storage_requirements = store
            .get_account_storage(account_id, AccountStorageFilter::All)
            .await
            .map(|storage| Self::build_storage_requirements(&storage))
            .unwrap_or_default();

        let known_code = store.get_account_code(account_id).await.ok().flatten();

        (storage_requirements, known_code)
    }

    /// Builds [`AccountStorageRequirements`] from [`AccountStorage`], requesting all entries for
    /// every map slot.
    fn build_storage_requirements(storage: &AccountStorage) -> AccountStorageRequirements {
        let map_slots = storage.slots().iter().filter_map(|slot: &StorageSlot| {
            if slot.slot_type() == StorageSlotType::Map {
                // Passing an empty key list requests all entries for this map slot.
                Some((slot.name().clone(), core::iter::empty::<&StorageMapKey>()))
            } else {
                None
            }
        });
        AccountStorageRequirements::new(map_slots)
    }

    /// Builds an [`Account`] directly from [`AccountDetails`] without any delta logic.
    ///
    /// This is used for accounts whose storage maps and vault fit within the node's size threshold.
    fn build_account_from_details(details: &AccountDetails) -> Result<Account, RpcError> {
        let mut slots: Vec<StorageSlot> = Vec::new();

        for slot_header in details.storage_details.header.slots() {
            match slot_header.slot_type() {
                StorageSlotType::Value => {
                    slots.push(StorageSlot::with_value(
                        slot_header.name().clone(),
                        slot_header.value(),
                    ));
                },
                StorageSlotType::Map => {
                    let map_details = details
                        .storage_details
                        .find_map_details(slot_header.name())
                        .ok_or_else(|| {
                            RpcError::ExpectedDataMissing(format!(
                                "slot '{}' is a map but has no map_details in response",
                                slot_header.name()
                            ))
                        })?;

                    let storage_map = map_details
                        .entries
                        .clone()
                        .into_storage_map()
                        .ok_or_else(|| {
                            RpcError::ExpectedDataMissing(
                                "expected AllEntries for full account fetch, got EntriesWithProofs"
                                    .into(),
                            )
                        })?
                        .map_err(|err| {
                            RpcError::InvalidResponse(format!(
                                "the rpc api returned a non-valid map entry: {err}"
                            ))
                        })?;

                    slots.push(StorageSlot::with_map(slot_header.name().clone(), storage_map));
                },
            }
        }

        let asset_vault = AssetVault::new(&details.vault_details.assets).map_err(|err| {
            RpcError::InvalidResponse(format!("rpc api returned non-valid assets: {err}"))
        })?;

        let account_storage = AccountStorage::new(slots).map_err(|err| {
            RpcError::InvalidResponse(format!("rpc api returned non-valid storage slots: {err}"))
        })?;

        Account::new(
            details.header.id(),
            asset_vault,
            account_storage,
            details.code.clone(),
            details.header.nonce(),
            None,
        )
        .map_err(|err| {
            RpcError::InvalidResponse(format!(
                "failed to construct account from rpc api response: {err}"
            ))
        })
    }

    /// Builds an [`AccountDelta`] from incremental RPC sync data, fetching local account
    /// data from the store on demand.
    ///
    /// For oversized storage maps: fetches delta entries via `sync_storage_maps`.
    /// For oversized vaults: fetches delta entries via `sync_account_vault`.
    /// Non-oversized parts are diffed against local data fetched from the store.
    ///
    /// # Panics
    ///
    /// Panics if `self.store` is `None`. Callers must check before invoking.
    #[allow(clippy::too_many_lines)]
    async fn build_account_delta(
        &self,
        details: &AccountDetails,
        local_header: &AccountHeader,
        sync_height: BlockNumber,
    ) -> Result<AccountDelta, ClientError> {
        let store = self.store.as_ref().expect("store required for delta sync");
        let account_id = details.header.id();

        let storage_delta = self
            .build_storage_delta(details, account_id, sync_height, store.as_ref())
            .await?;

        let vault_delta =
            self.build_vault_delta(details, account_id, sync_height, store.as_ref()).await?;

        // --- Nonce delta ---
        let old_nonce = local_header.nonce().as_canonical_u64();
        let new_nonce = details.header.nonce().as_canonical_u64();
        let nonce_delta = Felt::new(new_nonce - old_nonce);

        AccountDelta::new(account_id, storage_delta, vault_delta, nonce_delta).map_err(|err| {
            ClientError::RpcError(RpcError::InvalidResponse(format!(
                "failed to construct account delta: {err}"
            )))
        })
    }

    /// Computes the full storage delta (value slots + map slots) for the account.
    ///
    /// For value slots, compares the response values against the local store. For map slots,
    /// oversized maps (`too_many_entries`) fetch incremental delta entries from the sync endpoint
    /// and deduplicate by key keeping the latest value; non-oversized maps diff the full response
    /// entries against the local store.
    async fn build_storage_delta(
        &self,
        details: &AccountDetails,
        account_id: AccountId,
        sync_height: BlockNumber,
        store: &dyn Store,
    ) -> Result<AccountStorageDelta, ClientError> {
        let mut storage_delta = AccountStorageDelta::new();

        for slot_header in details.storage_details.header.slots() {
            if slot_header.slot_type() == StorageSlotType::Value {
                let local_value = store
                    .get_account_storage_item(account_id, slot_header.name().clone())
                    .await
                    .ok();

                if local_value.as_ref() != Some(&slot_header.value()) {
                    storage_delta
                        .set_item(slot_header.name().clone(), slot_header.value())
                        .map_err(|err| {
                            ClientError::RpcError(RpcError::InvalidResponse(format!(
                                "failed to set storage delta item: {err}"
                            )))
                        })?;
                }
            }
        }

        let mut map_delta_cache: Option<Vec<StorageMapUpdate>> = None;

        for slot_header in details.storage_details.header.slots() {
            if slot_header.slot_type() != StorageSlotType::Map {
                continue;
            }

            let map_details =
                details.storage_details.find_map_details(slot_header.name()).ok_or_else(|| {
                    ClientError::RpcError(RpcError::ExpectedDataMissing(format!(
                        "slot '{}' is a map but has no map_details in response",
                        slot_header.name()
                    )))
                })?;

            if map_details.too_many_entries {
                // Oversized map: fetch delta entries from the sync endpoint.
                if map_delta_cache.is_none() {
                    let map_info = self
                        .rpc_api
                        .sync_storage_maps(sync_height, None, account_id)
                        .await
                        .map_err(ClientError::RpcError)?;
                    map_delta_cache = Some(map_info.updates);
                }

                Self::apply_oversized_map_delta(
                    map_delta_cache.as_deref().unwrap_or_default(),
                    slot_header.name(),
                    &mut storage_delta,
                )?;
            } else {
                Self::apply_full_map_delta(
                    map_details,
                    slot_header.name(),
                    account_id,
                    store,
                    &mut storage_delta,
                )
                .await?;
            }
        }

        Ok(storage_delta)
    }

    /// Applies delta updates from the sync endpoint for an oversized storage map slot.
    ///
    /// Filters the cached delta updates to the target slot, sorts by block number, and
    /// deduplicates by key (keeping the latest value).
    fn apply_oversized_map_delta(
        delta_updates: &[StorageMapUpdate],
        slot_name: &StorageSlotName,
        storage_delta: &mut AccountStorageDelta,
    ) -> Result<(), ClientError> {
        let mut relevant: Vec<_> =
            delta_updates.iter().filter(|u| u.slot_name == *slot_name).collect();
        relevant.sort_by_key(|u| u.block_num);

        // Deduplicate: keep latest value per key.
        let mut seen = BTreeMap::new();
        for update in relevant {
            seen.insert(update.key, update.value);
        }

        for (key, value) in seen {
            storage_delta.set_map_item(slot_name.clone(), key, value).map_err(|err| {
                ClientError::RpcError(RpcError::InvalidResponse(format!(
                    "failed to set storage map delta: {err}"
                )))
            })?;
        }

        Ok(())
    }

    /// Diffs the full response map entries against the local store for a non-oversized map slot.
    ///
    /// Entries present in the response but missing or different locally are added to the delta.
    /// Entries present locally but absent in the response are set to `Word::default()` (removal).
    async fn apply_full_map_delta(
        map_details: &AccountStorageMapDetails,
        slot_name: &StorageSlotName,
        account_id: AccountId,
        store: &dyn Store,
        storage_delta: &mut AccountStorageDelta,
    ) -> Result<(), ClientError> {
        let response_map = map_details
            .entries
            .clone()
            .into_storage_map()
            .ok_or_else(|| {
                ClientError::RpcError(RpcError::ExpectedDataMissing(
                    "expected AllEntries for map, got EntriesWithProofs".into(),
                ))
            })?
            .map_err(|err| {
                ClientError::RpcError(RpcError::InvalidResponse(format!(
                    "the rpc api returned a non-valid map entry: {err}"
                )))
            })?;

        let local_entries: BTreeMap<StorageMapKey, Word> = store
            .get_account_storage(account_id, AccountStorageFilter::SlotName(slot_name.clone()))
            .await
            .ok()
            .and_then(|storage| storage.get(slot_name).cloned())
            .map(|slot| match slot.content() {
                StorageSlotContent::Map(map) => map.entries().map(|(k, v)| (*k, *v)).collect(),
                StorageSlotContent::Value(_) => BTreeMap::new(),
            })
            .unwrap_or_default();

        let response_entries: BTreeMap<StorageMapKey, Word> =
            response_map.entries().map(|(k, v)| (*k, *v)).collect();

        // Entries in response but not in local, or with different values.
        for (key, value) in &response_entries {
            if local_entries.get(key) != Some(value) {
                storage_delta.set_map_item(slot_name.clone(), *key, *value).map_err(|err| {
                    ClientError::RpcError(RpcError::InvalidResponse(format!(
                        "failed to set storage map delta: {err}"
                    )))
                })?;
            }
        }

        // Entries in local but removed in response (set to empty word).
        for key in local_entries.keys() {
            if !response_entries.contains_key(key) {
                storage_delta.set_map_item(slot_name.clone(), *key, Word::default()).map_err(
                    |err| {
                        ClientError::RpcError(RpcError::InvalidResponse(format!(
                            "failed to set storage map delta for removal: {err}"
                        )))
                    },
                )?;
            }
        }

        Ok(())
    }

    /// Computes the vault delta between local and remote account state.
    ///
    /// For oversized vaults (`too_many_assets`), fetches incremental updates from the sync
    /// endpoint and replays them on top of the local vault. For non-oversized vaults, diffs
    /// the full response assets against the local vault.
    async fn build_vault_delta(
        &self,
        details: &AccountDetails,
        account_id: AccountId,
        sync_height: BlockNumber,
        store: &dyn Store,
    ) -> Result<AccountVaultDelta, ClientError> {
        let mut vault_delta = AccountVaultDelta::default();
        let local_vault =
            store.get_account_vault(account_id).await.map_err(ClientError::StoreError)?;

        if details.vault_details.too_many_assets {
            // Oversized vault: fetch delta from sync endpoint.
            let vault_info = self
                .rpc_api
                .sync_account_vault(sync_height, None, account_id)
                .await
                .map_err(ClientError::RpcError)?;

            // Build the final vault state by applying updates to local vault.
            let mut vault_map: BTreeMap<AssetVaultKey, Asset> =
                local_vault.assets().map(|asset| (asset.vault_key(), asset)).collect();

            let mut vault_updates = vault_info.updates;
            vault_updates.sort_by_key(|u| u.block_num);

            for update in vault_updates {
                match update.asset {
                    Some(asset) => {
                        vault_map.insert(update.vault_key, asset);
                    },
                    None => {
                        vault_map.remove(&update.vault_key);
                    },
                }
            }

            Self::compute_vault_delta_from_diff(&local_vault, &vault_map, &mut vault_delta)?;
        } else {
            // Non-oversized vault: diff response assets against local.
            let final_assets: BTreeMap<AssetVaultKey, Asset> = details
                .vault_details
                .assets
                .iter()
                .map(|asset| (asset.vault_key(), *asset))
                .collect();

            Self::compute_vault_delta_from_diff(&local_vault, &final_assets, &mut vault_delta)?;
        }

        Ok(vault_delta)
    }

    /// Computes a vault delta from the difference between a local vault and a final asset map.
    fn compute_vault_delta_from_diff(
        local_vault: &AssetVault,
        final_assets: &BTreeMap<AssetVaultKey, Asset>,
        vault_delta: &mut AccountVaultDelta,
    ) -> Result<(), ClientError> {
        let local_assets: BTreeMap<AssetVaultKey, Asset> =
            local_vault.assets().map(|a| (a.vault_key(), a)).collect();

        // Assets in final but not in local -> add. Changed amounts -> remove old, add new.
        for (key, final_asset) in final_assets {
            match local_assets.get(key) {
                None => {
                    vault_delta.add_asset(*final_asset).map_err(|err| {
                        ClientError::RpcError(RpcError::InvalidResponse(format!(
                            "failed to add asset to vault delta: {err}"
                        )))
                    })?;
                },
                Some(local_asset) if local_asset != final_asset => {
                    vault_delta.remove_asset(*local_asset).map_err(|err| {
                        ClientError::RpcError(RpcError::InvalidResponse(format!(
                            "failed to remove old asset from vault delta: {err}"
                        )))
                    })?;
                    vault_delta.add_asset(*final_asset).map_err(|err| {
                        ClientError::RpcError(RpcError::InvalidResponse(format!(
                            "failed to add new asset to vault delta: {err}"
                        )))
                    })?;
                },
                _ => {}, // No change
            }
        }

        // Assets in local but not in final -> remove.
        for (key, local_asset) in &local_assets {
            if !final_assets.contains_key(key) {
                vault_delta.remove_asset(*local_asset).map_err(|err| {
                    ClientError::RpcError(RpcError::InvalidResponse(format!(
                        "failed to remove asset from vault delta: {err}"
                    )))
                })?;
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
            let external_consumer_account = state_sync_update
                .transaction_updates
                .external_nullifier_account(&nullifier_update.nullifier);

            state_sync_update.note_updates.apply_nullifiers_state_transitions(
                &nullifier_update,
                state_sync_update.transaction_updates.committed_transactions(),
                external_consumer_account,
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
                output_notes: vec![],
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
            StateSync::new(Arc::new(mock_rpc.clone()), None, Arc::new(CommitAllScreener), None);

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
            accounts: vec![AccountHeader::from(account)],
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

        let state_sync =
            StateSync::new(Arc::new(mock_rpc.clone()), None, Arc::new(MockScreener), None);

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
        let state_sync =
            StateSync::new(Arc::new(mock_rpc.clone()), None, Arc::new(MockScreener), None);

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
}
