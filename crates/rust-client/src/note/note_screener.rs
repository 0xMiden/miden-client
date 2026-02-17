use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec::Vec;

use async_trait::async_trait;
use miden_protocol::account::{Account, AccountId};
use miden_protocol::errors::{AccountError, AssetError};
use miden_protocol::note::{Note, NoteId};
use miden_standards::account::interface::{AccountInterface, AccountInterfaceExt};
use miden_standards::note::NoteConsumptionStatus;
use miden_tx::auth::TransactionAuthenticator;
use miden_tx::{NoteCheckerError, NoteConsumptionChecker, TransactionExecutor};
use thiserror::Error;

use crate::ClientError;
use crate::rpc::domain::note::CommittedNote;
use crate::store::data_store::ClientDataStore;
use crate::store::{InputNoteRecord, NoteFilter, Store, StoreError};
use crate::sync::{NoteUpdateAction, OnNoteReceived};
use crate::transaction::{InputNote, TransactionRequestBuilder, TransactionRequestError};

/// Represents the consumability of a note by a specific account.
///
/// The tuple contains the account ID that may consume the note and the moment it will become
/// relevant.
pub type NoteConsumability = (AccountId, NoteConsumptionStatus);

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
        let mut note_relevances = vec![];
        for id in self.store.get_account_ids().await? {
            let account_record = self
                .store
                .get_account(id)
                .await?
                .ok_or(NoteScreenerError::AccountDataNotFound(id))?;
            let account: Account = account_record
                .try_into()
                .map_err(|_| NoteScreenerError::AccountDataNotFound(id))?;

            match self.check_standard_consumability(&account, note).await? {
                NoteConsumptionStatus::NeverConsumable(_)
                | NoteConsumptionStatus::UnconsumableConditions => {},
                relevance => {
                    note_relevances.push((id, relevance));
                },
            }
        }

        Ok(note_relevances)
    }

    /// Tries to execute a standard consume transaction to check if the note is consumable by the
    /// account.
    pub async fn check_standard_consumability(
        &self,
        account: &Account,
        note: &Note,
    ) -> Result<NoteConsumptionStatus, NoteScreenerError> {
        let transaction_request =
            TransactionRequestBuilder::new().build_consume_notes(vec![note.clone()])?;

        let tx_script = transaction_request
            .build_transaction_script(&AccountInterface::from_account(account))?;

        let tx_args = transaction_request.clone().into_transaction_args(tx_script);

        let data_store = ClientDataStore::new(self.store.clone());
        let mut transaction_executor = TransactionExecutor::new(&data_store);
        if let Some(authenticator) = &self.authenticator {
            transaction_executor = transaction_executor.with_authenticator(authenticator.as_ref());
        }

        let consumption_checker = NoteConsumptionChecker::new(&transaction_executor);

        data_store.mast_store().load_account_code(account.code());
        let note_consumption_check = consumption_checker
            .can_consume(
                account.id(),
                self.store.get_sync_height().await?,
                InputNote::unauthenticated(note.clone()),
                tx_args,
            )
            .await?;

        Ok(note_consumption_check)
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
    #[error("failed to process note inputs")]
InvalidNoteInputsError(#[from] InvalidNoteInputsError),
    #[error("cannot check note relevance: account {0} data not found in the store")]
    AccountDataNotFound(AccountId),
    #[error("failed to fetch data from the store")]
    StoreError(#[from] StoreError),
    #[error("note consumption check failed")]
    NoteCheckerError(#[from] NoteCheckerError),
    #[error("failed to build transaction request")]
    TransactionRequestError(#[from] TransactionRequestError),
}

#[derive(Debug, Error)]
pub enum InvalidNoteInputsError {
    #[error("account error for note with id {0}: {1}")]
    AccountError(NoteId, AccountError),
    #[error("asset error for note with id {0}: {1}")]
    AssetError(NoteId, AssetError),
    #[error("expected {1} note inputs for note with id {0}")]
    WrongNumInputs(NoteId, usize),
    #[error("note input representing block with value {1} for note with id {0}")]
    BlockNumberError(NoteId, u64),
}
