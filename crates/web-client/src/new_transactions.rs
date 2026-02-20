use miden_client::ClientError;
use miden_client::asset::FungibleAsset;
use miden_client::note::{BlockNumber, Note as NativeNote};
use miden_client::transaction::{
    PaymentNoteDescription,
    ProvenTransaction as NativeProvenTransaction,
    SwapTransactionData,
    TransactionExecutorError,
    TransactionRequest as NativeTransactionRequest,
    TransactionRequestBuilder as NativeTransactionRequestBuilder,
    TransactionStoreUpdate as NativeTransactionStoreUpdate,
    TransactionSummary as NativeTransactionSummary,
};

use crate::prelude::*;
use crate::models::NoteType;
use crate::models::account_id::AccountId;
use crate::models::note::Note;
use crate::models::proven_transaction::ProvenTransaction;
use crate::models::provers::TransactionProver;
use crate::models::transaction_id::TransactionId;
use crate::models::transaction_request::TransactionRequest;
use crate::models::transaction_result::TransactionResult;
use crate::models::transaction_store_update::TransactionStoreUpdate;
use crate::models::transaction_summary::TransactionSummary;
use crate::WebClient;

// Internal helper: prove_transaction_impl (different prove wrapping per platform)
impl WebClient {
    async fn prove_transaction_impl(
        &self,
        transaction_result: &TransactionResult,
        prover: Option<TransactionProver>,
    ) -> platform::JsResult<ProvenTransaction> {
        let mut guard = lock_client!(self);
        let client = guard
            .as_mut()
            .ok_or_else(|| platform::error_from_string("Client not initialized"))?;

        let prover_arc =
            prover.map_or_else(|| client.prover(), |custom_prover| custom_prover.get_prover());

        #[cfg(feature = "wasm")]
        let result =
            Box::pin(client.prove_transaction_with(transaction_result.native(), prover_arc)).await;

        // SAFETY: The TransactionProver trait object held in the Arc is not Send+Sync by default,
        // but our concrete implementations (LocalTransactionProver, RemoteTransactionProver) are
        // Send-safe. We use assert_send to satisfy napi's Send requirement on async futures.
        #[cfg(feature = "napi")]
        let result = unsafe {
            crate::assert_send(Box::pin(
                client.prove_transaction_with(transaction_result.native(), prover_arc),
            ))
        }
        .await;

        result
            .map(Into::into)
            .map_err(|err| platform::error_with_context(err, "failed to prove transaction"))
    }
}

// Shared methods
#[bindings]
impl WebClient {
    /// Executes a transaction specified by the request against the specified account,
    /// proves it, submits it to the network, and updates the local database.
    ///
    /// Uses the prover configured for this client.
    #[bindings(js_name = "submitNewTransaction")]
    pub async fn submit_new_transaction(
        &self,
        account_id: &AccountId,
        transaction_request: &TransactionRequest,
    ) -> platform::JsResult<TransactionId> {
        let transaction_result =
            self.execute_transaction(account_id, transaction_request).await?;

        let tx_id = transaction_result.id();

        let proven_transaction = self.prove_transaction_impl(&transaction_result, None).await?;

        let submission_height =
            self.submit_proven_transaction(&proven_transaction, &transaction_result).await?;
        self.apply_transaction(&transaction_result, submission_height).await?;

        Ok(tx_id)
    }

    /// Executes a transaction specified by the request against the specified account, proves it
    /// with the user provided prover, submits it to the network, and updates the local database.
    #[bindings(js_name = "submitNewTransactionWithProver")]
    pub async fn submit_new_transaction_with_prover(
        &self,
        account_id: &AccountId,
        transaction_request: &TransactionRequest,
        prover: &TransactionProver,
    ) -> platform::JsResult<TransactionId> {
        let transaction_result =
            self.execute_transaction(account_id, transaction_request).await?;

        let tx_id = transaction_result.id();

        let proven_transaction = self
            .prove_transaction_impl(&transaction_result, Some(prover.clone()))
            .await?;

        let submission_height =
            self.submit_proven_transaction(&proven_transaction, &transaction_result).await?;
        self.apply_transaction(&transaction_result, submission_height).await?;

        Ok(tx_id)
    }

    /// Executes a transaction specified by the request against the specified account but does not
    /// submit it to the network nor update the local database. The returned [`TransactionResult`]
    /// retains the execution artifacts needed to continue with the transaction lifecycle.
    #[bindings(js_name = "executeTransaction")]
    pub async fn execute_transaction(
        &self,
        account_id: &AccountId,
        transaction_request: &TransactionRequest,
    ) -> platform::JsResult<TransactionResult> {
        let mut guard = lock_client!(self);
        let client = guard
            .as_mut()
            .ok_or_else(|| platform::error_from_string("Client not initialized"))?;

        Box::pin(client.execute_transaction(account_id.into(), transaction_request.into()))
            .await
            .map(TransactionResult::from)
            .map_err(|err| platform::error_with_context(err, "failed to execute transaction"))
    }

    /// Executes a transaction and returns the `TransactionSummary`.
    ///
    /// If the transaction is unauthorized (auth script emits the unauthorized event),
    /// returns the summary from the error. If the transaction succeeds, constructs
    /// a summary from the executed transaction using the `auth_arg` from the transaction
    /// request as the salt (or a zero salt if not provided).
    #[bindings(js_name = "executeForSummary")]
    pub async fn execute_for_summary(
        &self,
        account_id: &AccountId,
        transaction_request: &TransactionRequest,
    ) -> platform::JsResult<TransactionSummary> {
        let mut guard = lock_client!(self);
        let client = guard
            .as_mut()
            .ok_or_else(|| platform::error_from_string("Client not initialized"))?;

        let native_request: NativeTransactionRequest = transaction_request.into();
        // auth_arg is passed to the auth procedure as the salt for the transaction summary
        // defaults to 0 if not provided.
        let salt = native_request.auth_arg().unwrap_or_default();

        match Box::pin(client.execute_transaction(account_id.into(), native_request)).await {
            Ok(res) => {
                // construct summary from executed transaction
                let executed_tx = res.executed_transaction();
                let summary = NativeTransactionSummary::new(
                    executed_tx.account_delta().clone(),
                    executed_tx.input_notes().clone(),
                    executed_tx.output_notes().clone(),
                    salt,
                );
                Ok(TransactionSummary::from(summary))
            },
            Err(ClientError::TransactionExecutorError(
                TransactionExecutorError::Unauthorized(summary),
            )) => Ok(TransactionSummary::from(*summary)),
            Err(err) => Err(platform::error_with_context(err, "failed to execute transaction")),
        }
    }

    #[bindings(js_name = "submitProvenTransaction")]
    pub async fn submit_proven_transaction(
        &self,
        proven_transaction: &ProvenTransaction,
        transaction_result: &TransactionResult,
    ) -> platform::JsResult<u32> {
        let mut guard = lock_client!(self);
        let client = guard
            .as_mut()
            .ok_or_else(|| platform::error_from_string("Client not initialized"))?;

        let native_proven: NativeProvenTransaction = proven_transaction.clone().into();
        client
            .submit_proven_transaction(native_proven, transaction_result.native())
            .await
            .map(|block_number| block_number.as_u32())
            .map_err(|err| {
                platform::error_with_context(err, "failed to submit proven transaction")
            })
    }

    #[bindings(js_name = "applyTransaction")]
    pub async fn apply_transaction(
        &self,
        transaction_result: &TransactionResult,
        submission_height: u32,
    ) -> platform::JsResult<TransactionStoreUpdate> {
        let mut guard = lock_client!(self);
        let client = guard
            .as_mut()
            .ok_or_else(|| platform::error_from_string("Client not initialized"))?;

        let update = Box::pin(client.get_transaction_store_update(
            transaction_result.native(),
            BlockNumber::from(submission_height),
        ))
        .await
        .map(TransactionStoreUpdate::from)
        .map_err(|err| platform::error_with_context(err, "failed to build transaction update"))?;

        let native_update: NativeTransactionStoreUpdate = (&update).into();
        Box::pin(client.apply_transaction_update(native_update))
            .await
            .map_err(|err| {
                platform::error_with_context(err, "failed to apply transaction result")
            })?;

        Ok(update)
    }

    #[bindings(js_name = "newMintTransactionRequest")]
    pub async fn new_mint_transaction_request(
        &self,
        target_account_id: &AccountId,
        faucet_id: &AccountId,
        note_type: NoteType,
        amount: i64,
    ) -> platform::JsResult<TransactionRequest> {
        let fungible_asset = FungibleAsset::new(faucet_id.into(), amount as u64)
            .map_err(|err| platform::error_with_context(err, "failed to create fungible asset"))?;

        let mut guard = lock_client!(self);
        let client = guard
            .as_mut()
            .ok_or_else(|| {
                platform::error_from_string(
                    "Client not initialized while generating transaction request",
                )
            })?;

        let mint_transaction_request = NativeTransactionRequestBuilder::new()
            .build_mint_fungible_asset(
                fungible_asset,
                target_account_id.into(),
                note_type.into(),
                client.rng(),
            )
            .map_err(|err| {
                platform::error_with_context(err, "failed to create mint transaction request")
            })?;

        Ok(mint_transaction_request.into())
    }

    #[bindings(js_name = "newSendTransactionRequest")]
    pub async fn new_send_transaction_request(
        &self,
        sender_account_id: &AccountId,
        target_account_id: &AccountId,
        faucet_id: &AccountId,
        note_type: NoteType,
        amount: i64,
        recall_height: Option<u32>,
        timelock_height: Option<u32>,
    ) -> platform::JsResult<TransactionRequest> {
        let mut guard = lock_client!(self);
        let client = guard
            .as_mut()
            .ok_or_else(|| {
                platform::error_from_string(
                    "Client not initialized while generating transaction request",
                )
            })?;

        let fungible_asset = FungibleAsset::new(faucet_id.into(), amount as u64)
            .map_err(|err| platform::error_with_context(err, "failed to create fungible asset"))?;

        let mut payment_description = PaymentNoteDescription::new(
            vec![fungible_asset.into()],
            sender_account_id.into(),
            target_account_id.into(),
        );

        if let Some(recall_height) = recall_height {
            payment_description =
                payment_description.with_reclaim_height(BlockNumber::from(recall_height));
        }

        if let Some(height) = timelock_height {
            payment_description =
                payment_description.with_timelock_height(BlockNumber::from(height));
        }

        let send_transaction_request = NativeTransactionRequestBuilder::new()
            .build_pay_to_id(payment_description, note_type.into(), client.rng())
            .map_err(|err| {
                platform::error_with_context(err, "failed to create send transaction request")
            })?;

        Ok(send_transaction_request.into())
    }

    #[bindings(js_name = "newSwapTransactionRequest")]
    pub async fn new_swap_transaction_request(
        &self,
        sender_account_id: &AccountId,
        offered_asset_faucet_id: &AccountId,
        offered_asset_amount: i64,
        requested_asset_faucet_id: &AccountId,
        requested_asset_amount: i64,
        note_type: NoteType,
        payback_note_type: NoteType,
    ) -> platform::JsResult<TransactionRequest> {
        let offered_fungible_asset =
            FungibleAsset::new(offered_asset_faucet_id.into(), offered_asset_amount as u64)
                .map_err(|err| {
                    platform::error_with_context(err, "failed to create offered fungible asset")
                })?
                .into();

        let requested_fungible_asset =
            FungibleAsset::new(requested_asset_faucet_id.into(), requested_asset_amount as u64)
                .map_err(|err| {
                    platform::error_with_context(err, "failed to create requested fungible asset")
                })?
                .into();

        let swap_transaction_data = SwapTransactionData::new(
            sender_account_id.into(),
            offered_fungible_asset,
            requested_fungible_asset,
        );

        let mut guard = lock_client!(self);
        let client = guard
            .as_mut()
            .ok_or_else(|| {
                platform::error_from_string(
                    "Client not initialized while generating transaction request",
                )
            })?;

        let swap_transaction_request = NativeTransactionRequestBuilder::new()
            .build_swap(
                &swap_transaction_data,
                note_type.into(),
                payback_note_type.into(),
                client.rng(),
            )
            .map_err(|err| {
                platform::error_with_context(err, "failed to create swap transaction request")
            })?;

        Ok(swap_transaction_request.into())
    }

    /// Generates a transaction proof using the client's default prover.
    #[bindings(js_name = "proveTransaction")]
    pub async fn prove_transaction(
        &self,
        transaction_result: &TransactionResult,
    ) -> platform::JsResult<ProvenTransaction> {
        self.prove_transaction_impl(transaction_result, None).await
    }

    /// Generates a transaction proof using the provided custom prover.
    #[bindings(js_name = "proveTransactionWithProver")]
    pub async fn prove_transaction_with_prover(
        &self,
        transaction_result: &TransactionResult,
        prover: &TransactionProver,
    ) -> platform::JsResult<ProvenTransaction> {
        self.prove_transaction_impl(transaction_result, Some(prover.clone()))
            .await
    }
}

// wasm-only: new_consume_transaction_request (takes owned Vec<Note>)
#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl WebClient {
    #[wasm_bindgen(js_name = "newConsumeTransactionRequest")]
    pub fn new_consume_transaction_request(
        &self,
        list_of_notes: Vec<Note>,
    ) -> platform::JsResult<TransactionRequest> {
        let native_notes = list_of_notes
            .into_iter()
            .map(NativeNote::try_from)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|err| {
                platform::error_from_string(&format!(
                    "Failed to convert note to native note: {err}"
                ))
            })?;

        let consume_transaction_request = NativeTransactionRequestBuilder::new()
            .build_consume_notes(native_notes)
            .map_err(|err| {
                platform::error_from_string(&format!(
                    "Failed to create Consume Transaction Request: {err}"
                ))
            })?;

        Ok(consume_transaction_request.into())
    }
}

// napi-only: new_consume_transaction_request (takes Vec<&Note>)
#[cfg(feature = "napi")]
#[napi_derive::napi]
impl WebClient {
    pub async fn new_consume_transaction_request(
        &self,
        list_of_notes: Vec<&Note>,
    ) -> platform::JsResult<TransactionRequest> {
        let native_notes: Vec<NativeNote> =
            list_of_notes.into_iter().map(NativeNote::from).collect();

        let consume_transaction_request = NativeTransactionRequestBuilder::new()
            .build_consume_notes(native_notes)
            .map_err(|err| {
                platform::error_from_string(&format!(
                    "Failed to create Consume Transaction Request: {err}"
                ))
            })?;

        Ok(consume_transaction_request.into())
    }
}
