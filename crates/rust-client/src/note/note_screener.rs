use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::fmt;

use async_trait::async_trait;
use miden_lib::account::interface::AccountInterface;
use miden_lib::note::well_known_note::WellKnownNote;
use miden_objects::account::{Account, AccountId};
use miden_objects::assembly::debuginfo::SourceManagerSync;
use miden_objects::note::{Note, NoteId};
use miden_objects::{AccountError, AssetError};
use miden_tx::auth::TransactionAuthenticator;
use miden_tx::{
    NoteCheckerError,
    NoteConsumptionChecker,
    NoteConsumptionStatus,
    TransactionExecutor,
};
use thiserror::Error;

use crate::ClientError;
use crate::rpc::NodeRpcClient;
use crate::rpc::domain::note::CommittedNote;
use crate::store::data_store::ClientDataStore;
use crate::store::{InputNoteRecord, NoteFilter, Store, StoreError};
use crate::sync::{NoteUpdateAction, OnNoteReceived};
use crate::transaction::{InputNote, TransactionRequestBuilder, TransactionRequestError};

/// Describes the relevance of a note based on the screening.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum NoteRelevance {
    /// The note can be consumed in the client's current block.
    Now,
    /// The note can be consumed after the block with the specified number.
    After(u32),
}

/// Represents the consumability of a note by a specific account.
///
/// The tuple contains the account ID that may consume the note and the moment it will become
/// relevant.
pub type NoteConsumability = (AccountId, NoteRelevance);

impl fmt::Display for NoteRelevance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NoteRelevance::Now => write!(f, "Now"),
            NoteRelevance::After(height) => write!(f, "After block {height}"),
        }
    }
}

/// Provides functionality for testing whether a note is relevant to the client or not.
///
/// Here, relevance is based on whether the note is able to be consumed by an account that is
/// tracked in the provided `store`. This can be derived in a number of ways, such as looking
/// at the combination of script root and note inputs. For example, a P2ID note is relevant
/// for a specific account ID if this ID is its first note input.
pub struct NoteScreener<AUTH> {
    /// A reference to the client's store, used to fetch necessary data to check consumability.
    store: Arc<dyn Store>,
    /// A reference to the transaction authenticator
    authenticator: Option<Arc<AUTH>>,
    /// Shared source manager used when compiling scripts.
    source_manager: Arc<dyn SourceManagerSync>,
    /// RPC client for fetching data from the node when not available locally.
    rpc_api: Arc<dyn NodeRpcClient>,
}

impl<AUTH> NoteScreener<AUTH>
where
    AUTH: TransactionAuthenticator + Sync,
{
    pub fn new(
        store: Arc<dyn Store>,
        authenticator: Option<Arc<AUTH>>,
        source_manager: Arc<dyn SourceManagerSync>,
        rpc_api: Arc<dyn NodeRpcClient>,
    ) -> Self {
        Self {
            store,
            authenticator,
            source_manager,
            rpc_api,
        }
    }

    /// Returns a vector of tuples describing the relevance of the provided note to the
    /// accounts monitored by this screener.
    ///
    /// If relevance can't be determined, the screener defaults to setting the note as consumable.
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

            match self.check_standard_consumability(account_record.account(), note).await {
                Ok(Some(relevance)) => {
                    note_relevances.push((id, relevance));
                },
                Ok(None) => {
                    // The note might be consumable after a certain block height if the note is
                    // p2ide
                    let script_root = note.script().root();

                    if script_root == WellKnownNote::P2IDE.script_root()
                        && let Some(relevance) = Self::check_p2ide_recall_consumability(note, &id)?
                    {
                        note_relevances.push((id, relevance));
                    }
                },
                // If an error occurs while checking consumability, we count it as not relevant for
                // that account
                Err(_) => {},
            }
        }

        Ok(note_relevances)
    }

    /// Tries to execute a standard consume transaction to check if the note is consumable by the
    /// account.
    async fn check_standard_consumability(
        &self,
        account: &Account,
        note: &Note,
    ) -> Result<Option<NoteRelevance>, NoteScreenerError> {
        let transaction_request =
            TransactionRequestBuilder::new().build_consume_notes(vec![note.id()])?;

        let tx_script = transaction_request.build_transaction_script(
            &AccountInterface::from(account),
            crate::DebugMode::Disabled,
        )?;

        let tx_args = transaction_request.clone().into_transaction_args(tx_script);

        let data_store = ClientDataStore::new(self.store.clone(), self.rpc_api.clone());
        let mut transaction_executor = TransactionExecutor::new(&data_store);
        if let Some(authenticator) = &self.authenticator {
            transaction_executor = transaction_executor.with_authenticator(authenticator.as_ref());
        }
        transaction_executor =
            transaction_executor.with_source_manager(self.source_manager.clone());

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

        let result = match note_consumption_check {
            NoteConsumptionStatus::ConsumableAfter(block_number) => {
                Some(NoteRelevance::After(block_number.as_u32()))
            },
            NoteConsumptionStatus::Consumable
            | NoteConsumptionStatus::ConsumableWithAuthorization => Some(NoteRelevance::Now),
            // NOTE: NoteConsumptionStatus::Unconsumable means that state-related context does not
            // allow for consumption, so don't keep for now. In the next version, we should be more
            // careful about this
            NoteConsumptionStatus::Unconsumable | NoteConsumptionStatus::Incompatible => None,
        };
        Ok(result)
    }

    /// Special relevance check for P2IDE notes. It checks if the sender account can consume and
    /// recall the note.
    fn check_p2ide_recall_consumability(
        note: &Note,
        account_id: &AccountId,
    ) -> Result<Option<NoteRelevance>, NoteScreenerError> {
        let note_inputs = note.inputs().values();
        if note_inputs.len() != 4 {
            return Err(InvalidNoteInputsError::WrongNumInputs(note.id(), 4).into());
        }

        let recall_height_felt = note_inputs[2];

        let sender = note.metadata().sender();
        let recall_height: u32 = recall_height_felt.as_int().try_into().map_err(|_err| {
            InvalidNoteInputsError::BlockNumberError(note.id(), recall_height_felt.as_int())
        })?;

        if sender == *account_id {
            Ok(Some(NoteRelevance::After(recall_height)))
        } else {
            Ok(None)
        }
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
    #[error("error while processing note inputs")]
    InvalidNoteInputsError(#[from] InvalidNoteInputsError),
    #[error("account data wasn't found for account id {0}")]
    AccountDataNotFound(AccountId),
    #[error("error while fetching data from the store")]
    StoreError(#[from] StoreError),
    #[error("error while checking note")]
    NoteCheckerError(#[from] NoteCheckerError),
    #[error("error while building transaction request")]
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
