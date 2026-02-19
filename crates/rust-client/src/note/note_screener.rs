use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec::Vec;

use async_trait::async_trait;
use miden_protocol::account::{Account, AccountCode, AccountId};
use miden_protocol::block::BlockNumber;
use miden_protocol::note::Note;
use miden_standards::note::{NoteConsumptionStatus, StandardNote};
use miden_tx::auth::TransactionAuthenticator;
use miden_tx::{
    NoteCheckerError,
    NoteConsumptionChecker,
    NoteConsumptionInfo,
    TransactionExecutor,
};
use thiserror::Error;

use crate::ClientError;
use crate::rpc::domain::note::CommittedNote;
use crate::store::data_store::ClientDataStore;
use crate::store::{InputNoteRecord, NoteFilter, Store, StoreError};
use crate::sync::{NoteUpdateAction, OnNoteReceived};
use crate::transaction::{AdviceMap, InputNote, TransactionArgs, TransactionRequestError};

/// Represents the consumability of a note by a specific account.
///
/// The tuple contains the account ID that may consume the note and the moment it will become
/// relevant.
pub type NoteConsumability = (AccountId, NoteConsumptionStatus);

/// Returns `true` if the consumption status indicates that the note may be consumable by the
/// account. A note is considered relevant unless it is permanently unconsumable (either due to
/// a fundamental incompatibility or unconsumable conditions).
fn is_relevant(consumption_status: &NoteConsumptionStatus) -> bool {
    !matches!(
        consumption_status,
        NoteConsumptionStatus::NeverConsumable(_) | NoteConsumptionStatus::UnconsumableConditions
    )
}

/// Provides functionality for testing whether a note is relevant to the client or not.
///
/// Here, relevance is based on whether the note is able to be consumed by an account that is
/// tracked in the provided `store`. This can be derived in a number of ways, such as looking
/// at the combination of script root and note inputs. For example, a P2ID note is relevant
/// for a specific account ID if this ID is its first note input.
#[derive(Clone)]
pub struct NoteScreener<AUTH> {
    /// A reference to the client's store, used to fetch necessary data to check consumability.
    store: Arc<dyn Store>,
    /// A reference to the transaction authenticator
    authenticator: Option<Arc<AUTH>>,
}

impl<AUTH> NoteScreener<AUTH>
where
    AUTH: TransactionAuthenticator + Sync,
{
    pub fn new(store: Arc<dyn Store>, authenticator: Option<Arc<AUTH>>) -> Self {
        Self { store, authenticator }
    }

    /// Returns a vector of tuples describing the relevance of the provided note to the
    /// accounts monitored by this screener.
    ///
    /// The relevance is determined by [`NoteConsumptionChecker::can_consume`] and is based on
    /// current conditions (for example, it takes the latest block in the client as reference).
    pub async fn check_relevance(
        &self,
        note: &Note,
    ) -> Result<Vec<NoteConsumability>, NoteScreenerError> {
        Ok(self
            .check_relevance_batch(core::slice::from_ref(note))
            .await?
            .pop()
            .unwrap_or_default())
    }

    /// Returns note relevances for a batch of notes, preserving input order.
    ///
    /// For each note, returns a list of `(AccountId, NoteConsumptionStatus)` pairs for all
    /// tracked accounts that could potentially consume it. Notes that are permanently
    /// unconsumable by an account (i.e., `NeverConsumable` or `UnconsumableConditions`) are
    /// filtered out from the results.
    pub async fn check_relevance_batch(
        &self,
        notes: &[Note],
    ) -> Result<Vec<Vec<NoteConsumability>>, NoteScreenerError> {
        let account_ids = self.store.get_account_ids().await?;
        if notes.is_empty() || account_ids.is_empty() {
            return Ok(vec![Vec::new(); notes.len()]);
        }

        let block_ref = self.store.get_sync_height().await?;
        let standard_notes = notes.iter().map(StandardNote::from_note).collect::<Vec<_>>();
        let mut note_relevances = vec![Vec::new(); notes.len()];
        let tx_args = TransactionArgs::new(AdviceMap::default());

        let data_store = ClientDataStore::new(self.store.clone());
        let mut transaction_executor = TransactionExecutor::new(&data_store);
        if let Some(authenticator) = &self.authenticator {
            transaction_executor = transaction_executor.with_authenticator(authenticator.as_ref());
        }
        let consumption_checker = NoteConsumptionChecker::new(&transaction_executor);

        for account_id in account_ids {
            let mut runtime_note_indices = Vec::new();

            for (note_idx, note) in notes.iter().enumerate() {
                if let Some(standard_note) = standard_notes[note_idx].as_ref()
                    && let Some(consumption_status) =
                        standard_note.is_consumable(note, account_id, block_ref)
                {
                    if is_relevant(&consumption_status) {
                        note_relevances[note_idx].push((account_id, consumption_status));
                    }
                    continue;
                }

                runtime_note_indices.push(note_idx);
            }

            if runtime_note_indices.is_empty() {
                continue;
            }

            let account_code = self.get_account_code(account_id).await?;
            data_store.mast_store().load_account_code(&account_code);

            for note_idx in runtime_note_indices {
                let consumption_status = consumption_checker
                    .can_consume(
                        account_id,
                        block_ref,
                        InputNote::unauthenticated(notes[note_idx].clone()),
                        tx_args.clone(),
                    )
                    .await?;

                if is_relevant(&consumption_status) {
                    note_relevances[note_idx].push((account_id, consumption_status));
                }
            }
        }

        Ok(note_relevances)
    }

    /// Runs note consumability checking for many notes at once using
    /// [`NoteConsumptionChecker::check_notes_consumability`].
    pub async fn check_notes_consumability(
        &self,
        account_id: AccountId,
        notes: Vec<Note>,
    ) -> Result<NoteConsumptionInfo, NoteScreenerError> {
        let block_ref = self.store.get_sync_height().await?;
        let tx_args = TransactionArgs::new(AdviceMap::default());
        let account_code = self.get_account_code(account_id).await?;

        let data_store = ClientDataStore::new(self.store.clone());
        let mut transaction_executor = TransactionExecutor::new(&data_store);
        if let Some(authenticator) = &self.authenticator {
            transaction_executor = transaction_executor.with_authenticator(authenticator.as_ref());
        }

        let consumption_checker = NoteConsumptionChecker::new(&transaction_executor);

        data_store.mast_store().load_account_code(&account_code);
        let note_consumption_info = consumption_checker
            .check_notes_consumability(account_id, block_ref, notes, tx_args)
            .await?;

        Ok(note_consumption_info)
    }

    /// Tries to execute a standard consume transaction to check if the note is consumable by the
    /// account.
    pub async fn check_standard_consumability(
        &self,
        account: &Account,
        note: &Note,
    ) -> Result<NoteConsumptionStatus, NoteScreenerError> {
        let block_ref = self.store.get_sync_height().await?;
        if let Some(standard_note) = StandardNote::from_note(note)
            && let Some(consumption_status) =
                standard_note.is_consumable(note, account.id(), block_ref)
        {
            return Ok(consumption_status);
        }

        let tx_args = TransactionArgs::new(AdviceMap::default());
        self.check_standard_consumability_for_account(
            account.id(),
            account.code(),
            note,
            block_ref,
            tx_args,
        )
        .await
    }

    async fn check_standard_consumability_for_account(
        &self,
        account_id: AccountId,
        account_code: &AccountCode,
        note: &Note,
        block_ref: BlockNumber,
        tx_args: TransactionArgs,
    ) -> Result<NoteConsumptionStatus, NoteScreenerError> {
        let data_store = ClientDataStore::new(self.store.clone());
        let mut transaction_executor = TransactionExecutor::new(&data_store);
        if let Some(authenticator) = &self.authenticator {
            transaction_executor = transaction_executor.with_authenticator(authenticator.as_ref());
        }

        let consumption_checker = NoteConsumptionChecker::new(&transaction_executor);

        data_store.mast_store().load_account_code(account_code);
        let note_consumption_check = consumption_checker
            .can_consume(account_id, block_ref, InputNote::unauthenticated(note.clone()), tx_args)
            .await?;

        Ok(note_consumption_check)
    }

    async fn get_account_code(
        &self,
        account_id: AccountId,
    ) -> Result<AccountCode, NoteScreenerError> {
        self.store
            .get_account_code(account_id)
            .await?
            .ok_or(NoteScreenerError::AccountDataNotFound(account_id))
    }
}

// DEFAULT CALLBACK IMPLEMENTATIONS
// ================================================================================================

#[async_trait(?Send)]
impl<AUTH> OnNoteReceived for NoteScreener<AUTH>
where
    AUTH: TransactionAuthenticator + Sync,
{
    /// Default implementation of the [`OnNoteReceived`] callback. It queries the store for the
    /// committed note to check if it's relevant. If the note wasn't being tracked but it came in
    /// the sync response it may be a new public note, in that case we use the [`NoteScreener`]
    /// to check its relevance.
    async fn on_note_received(
        &self,
        committed_note: CommittedNote,
        public_note: Option<InputNoteRecord>,
    ) -> Result<NoteUpdateAction, ClientError> {
        let note_id = *committed_note.note_id();

        let input_note_present =
            !self.store.get_input_notes(NoteFilter::Unique(note_id)).await?.is_empty();
        let output_note_present =
            !self.store.get_output_notes(NoteFilter::Unique(note_id)).await?.is_empty();

        if input_note_present || output_note_present {
            // The note is being tracked by the client so it is relevant
            return Ok(NoteUpdateAction::Commit(committed_note));
        }

        match public_note {
            Some(public_note) => {
                // If tracked by the user, keep note regardless of inputs and extra checks
                if let Some(metadata) = public_note.metadata()
                    && self.store.get_unique_note_tags().await?.contains(&metadata.tag())
                {
                    return Ok(NoteUpdateAction::Insert(public_note));
                }

                // The note is not being tracked by the client and is public so we can screen it
                let new_note_relevance = self
                    .check_relevance(
                        &public_note
                            .clone()
                            .try_into()
                            .map_err(ClientError::NoteRecordConversionError)?,
                    )
                    .await?;
                let is_relevant = !new_note_relevance.is_empty();
                if is_relevant {
                    Ok(NoteUpdateAction::Insert(public_note))
                } else {
                    Ok(NoteUpdateAction::Discard)
                }
            },
            None => {
                // The note is not being tracked by the client and is private so we can't determine
                // if it is relevant
                Ok(NoteUpdateAction::Discard)
            },
        }
    }
}

// NOTE SCREENER ERRORS
// ================================================================================================

/// Error when screening notes to check relevance to a client.
#[derive(Debug, Error)]
pub enum NoteScreenerError {
    #[error("account data wasn't found for account id {0}")]
    AccountDataNotFound(AccountId),
    #[error("error while fetching data from the store")]
    StoreError(#[from] StoreError),
    #[error("error while checking note")]
    NoteCheckerError(#[from] NoteCheckerError),
    #[error("error while building transaction request")]
    TransactionRequestError(#[from] TransactionRequestError),
}
