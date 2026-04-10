use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use miden_protocol::Word;
use miden_protocol::account::{AccountHeader, AccountId};
use miden_protocol::block::{BlockHeader, BlockNumber};
use miden_protocol::crypto::merkle::mmr::{MmrDelta, PartialMmr};
use miden_protocol::note::{Note, NoteId, NoteType};

use super::{BlockUpdates, NoteUpdateAction, StateSync, TransactionUpdateTracker};
use crate::ClientError;
use crate::note::NoteUpdateTracker;
use crate::rpc::domain::note::{CommittedNote, NoteSyncBlock};
use crate::rpc::domain::nullifier::NullifierUpdate;
use crate::store::{InputNoteRecord, StoreError};
use crate::sync::AccountUpdates;

// APPLY METHODS
// ================================================================================================

impl StateSync {
    /// Advances the partial MMR to the chain tip.
    ///
    /// Applies the MMR delta (covering blocks up to `chain_tip - 1`), then adds the chain tip
    /// leaf (which the delta excludes due to the one-block lag in block header MMR commitments).
    pub(super) fn advance_mmr(
        mmr_delta: MmrDelta,
        chain_tip_header: &BlockHeader,
        current_partial_mmr: &mut PartialMmr,
    ) -> Result<BlockUpdates, ClientError> {
        let mut new_authentication_nodes =
            current_partial_mmr.apply(mmr_delta).map_err(StoreError::MmrError)?;
        let new_peaks = current_partial_mmr.peaks();
        new_authentication_nodes
            .append(&mut current_partial_mmr.add(chain_tip_header.commitment(), false));

        let mut block_updates = BlockUpdates::default();
        block_updates.insert(chain_tip_header.clone(), false, new_peaks, new_authentication_nodes);

        Ok(block_updates)
    }

    /// Screens note inclusions and tracks relevant blocks in the MMR.
    ///
    /// For each block with note inclusions:
    /// 1. Builds public note records from fetched note bodies.
    /// 2. Screens each note via the [`OnNoteReceived`] callback.
    /// 3. If a relevant note is found, tracks that block in the partial MMR.
    pub(super) async fn process_note_inclusions(
        &self,
        note_blocks: Vec<NoteSyncBlock>,
        public_notes: BTreeMap<NoteId, Note>,
        block_updates: &mut BlockUpdates,
        note_updates: &mut NoteUpdateTracker,
        current_partial_mmr: &mut PartialMmr,
    ) -> Result<(), ClientError> {
        // Build input note records for public notes from the fetched note bodies and the
        // inclusion proofs already present in the note blocks.
        let mut public_note_records: BTreeMap<NoteId, InputNoteRecord> = BTreeMap::new();
        for (note_id, note) in public_notes {
            let inclusion_proof = note_blocks
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

        for block in note_blocks {
            let found_relevant_note = self
                .screen_notes(note_updates, block.notes, &block.block_header, &public_note_records)
                .await?;

            if found_relevant_note {
                Self::track_block_in_mmr(
                    &block.block_header,
                    &block.mmr_path,
                    block_updates,
                    current_partial_mmr,
                )?;
            }
        }

        Ok(())
    }

    /// Updates account states by comparing local state with the node.
    ///
    /// Fetches the latest state for public accounts that have changed, and flags private accounts
    /// whose commitment doesn't match the node (potentially locked).
    pub(super) async fn process_account_updates(
        &self,
        accounts: &[AccountHeader],
        account_commitment_updates: &[(AccountId, Word)],
    ) -> Result<AccountUpdates, ClientError> {
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

        Ok(AccountUpdates::new(updated_public_accounts, mismatched_private_accounts))
    }

    /// Detects notes consumed externally by querying the node for new nullifiers.
    ///
    /// Queries `sync_nullifiers` for unspent note prefixes, then applies state transitions:
    /// notes are marked as consumed (locally or externally) and pending transactions whose
    /// input notes were nullified are discarded.
    pub(super) async fn sync_consumed_notes(
        &self,
        chain_tip_block_num: BlockNumber,
        current_block_num: BlockNumber,
        note_updates: &mut NoteUpdateTracker,
        transaction_updates: &mut TransactionUpdateTracker,
    ) -> Result<(), ClientError> {
        let nullifiers_tags: Vec<u16> =
            note_updates.unspent_nullifiers().map(|nullifier| nullifier.prefix()).collect();

        let mut new_nullifiers = self
            .rpc_api
            .sync_nullifiers(&nullifiers_tags, current_block_num, Some(chain_tip_block_num))
            .await?;

        // Discard nullifiers newer than the synced block (possible if the chain tip advances
        // between the sync_state and sync_nullifiers calls).
        new_nullifiers.retain(|n| n.block_num <= chain_tip_block_num);

        apply_nullifier_updates(new_nullifiers, note_updates, transaction_updates)
    }

    // PRIVATE HELPERS
    // --------------------------------------------------------------------------------------------

    /// Screens note inclusions from a single block using the [`OnNoteReceived`] callback.
    ///
    /// Returns `true` if at least one relevant note was found (meaning the block should be
    /// tracked in the MMR).
    async fn screen_notes(
        &self,
        note_updates: &mut NoteUpdateTracker,
        note_inclusions: BTreeMap<NoteId, CommittedNote>,
        block_header: &BlockHeader,
        public_notes: &BTreeMap<NoteId, InputNoteRecord>,
    ) -> Result<bool, ClientError> {
        let mut found_relevant_note = false;

        for (_, committed_note) in note_inclusions {
            let public_note = (committed_note.note_type() != NoteType::Private)
                .then(|| public_notes.get(committed_note.note_id()))
                .flatten()
                .cloned();

            match self.note_screener.on_note_received(committed_note, public_note).await? {
                NoteUpdateAction::Commit(committed_note) => {
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

    /// Tracks a block containing relevant notes in the partial MMR.
    fn track_block_in_mmr(
        block_header: &BlockHeader,
        mmr_path: &miden_protocol::crypto::merkle::MerklePath,
        block_updates: &mut BlockUpdates,
        current_partial_mmr: &mut PartialMmr,
    ) -> Result<(), ClientError> {
        let block_pos = block_header.block_num().as_usize();

        let track_auth_nodes = if current_partial_mmr.is_tracked(block_pos) {
            vec![]
        } else {
            let nodes_before: BTreeMap<_, _> =
                current_partial_mmr.nodes().map(|(k, v)| (*k, *v)).collect();
            current_partial_mmr
                .track(block_pos, block_header.commitment(), mmr_path)
                .map_err(StoreError::MmrError)?;
            current_partial_mmr
                .nodes()
                .filter(|(k, _)| !nodes_before.contains_key(k))
                .map(|(k, v)| (*k, *v))
                .collect()
        };

        block_updates.insert(
            block_header.clone(),
            true,
            current_partial_mmr.peaks(),
            track_auth_nodes,
        );

        Ok(())
    }
}

// HELPERS
// ================================================================================================

/// Applies nullifier updates to both note and transaction trackers.
///
/// For each nullifier:
/// 1. Resolves whether a committed local transaction consumed the note.
/// 2. Resolves the external consumer account (if any) from the transaction tracker.
/// 3. Applies the state transition to the note tracker.
/// 4. Discards pending transactions whose input note was nullified.
fn apply_nullifier_updates(
    nullifier_updates: Vec<NullifierUpdate>,
    note_updates: &mut NoteUpdateTracker,
    transaction_updates: &mut TransactionUpdateTracker,
) -> Result<(), ClientError> {
    for update in nullifier_updates {
        let external_consumer = transaction_updates.external_nullifier_account(&update.nullifier);
        let consumed_tx_order = transaction_updates.nullifier_order(&update.nullifier);

        note_updates.apply_nullifiers_state_transitions(
            &update,
            |tx_id| transaction_updates.committed_transaction_block(tx_id),
            external_consumer,
            consumed_tx_order,
        )?;

        transaction_updates.apply_input_note_nullified(update.nullifier);
    }
    Ok(())
}
