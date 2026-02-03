use miden_client::ClientError;
use miden_client::asset::FungibleAsset;
use miden_client::note::{BlockNumber, Note as NativeNote};
use miden_client::transaction::{
    PaymentNoteDescription, ProvenTransaction as NativeProvenTransaction, SwapTransactionData,
    TransactionExecutorError, TransactionRequest as NativeTransactionRequest,
    TransactionRequestBuilder as NativeTransactionRequestBuilder,
    TransactionStoreUpdate as NativeTransactionStoreUpdate,
    TransactionSummary as NativeTransactionSummary,
};
use wasm_bindgen::prelude::*;

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
use crate::{WebClient, js_error_with_context};

#[wasm_bindgen]
impl WebClient {
    /// Executes a transaction specified by the request against the specified account,
    /// proves it, submits it to the network, and updates the local database.
    ///
    /// Uses the prover configured for this client.
    ///
    /// If the transaction utilizes foreign account data, there is a chance that the client doesn't
    /// have the required block header in the local database. In these scenarios, a sync to
    /// the chain tip is performed, and the required block header is retrieved.
    #[wasm_bindgen(js_name = "submitNewTransaction")]
    pub async fn submit_new_transaction(
        &mut self,
        account_id: &AccountId,
        transaction_request: &TransactionRequest,
    ) -> Result<TransactionId, JsValue> {
        let transaction_result = self.execute_transaction(account_id, transaction_request).await?;

        let tx_id = transaction_result.id();

        let proven_transaction = self.prove_transaction(&transaction_result, None).await?;

        let submission_height =
            self.submit_proven_transaction(&proven_transaction, &transaction_result).await?;
        self.apply_transaction(&transaction_result, submission_height).await?;

        Ok(tx_id)
    }

    /// Executes a transaction specified by the request against the specified account, proves it
    /// with the user provided prover, submits it to the network, and updates the local database.
    ///
    /// If the transaction utilizes foreign account data, there is a chance that the client doesn't
    /// have the required block header in the local database. In these scenarios, a sync to the
    /// chain tip is performed, and the required block header is retrieved.
    #[wasm_bindgen(js_name = "submitNewTransactionWithProver")]
    pub async fn submit_new_transaction_with_prover(
        &mut self,
        account_id: &AccountId,
        transaction_request: &TransactionRequest,
        prover: &TransactionProver,
    ) -> Result<TransactionId, JsValue> {
        let transaction_result = self.execute_transaction(account_id, transaction_request).await?;

        let tx_id = transaction_result.id();

        let proven_transaction =
            self.prove_transaction(&transaction_result, Some(prover.clone())).await?;

        let submission_height =
            self.submit_proven_transaction(&proven_transaction, &transaction_result).await?;
        self.apply_transaction(&transaction_result, submission_height).await?;

        Ok(tx_id)
    }

    /// Executes a transaction specified by the request against the specified account but does not
    /// submit it to the network nor update the local database. The returned [`TransactionResult`]
    /// retains the execution artifacts needed to continue with the transaction lifecycle.
    ///
    /// If the transaction utilizes foreign account data, there is a chance that the client doesn't
    /// have the required block header in the local database. In these scenarios, a sync to
    /// the chain tip is performed, and the required block header is retrieved.
    #[wasm_bindgen(js_name = "executeTransaction")]
    pub async fn execute_transaction(
        &mut self,
        account_id: &AccountId,
        transaction_request: &TransactionRequest,
    ) -> Result<TransactionResult, JsValue> {
        if let Some(client) = self.get_mut_inner() {
            Box::pin(client.execute_transaction(account_id.into(), transaction_request.into()))
                .await
                .map(TransactionResult::from)
                .map_err(|err| js_error_with_context(err, "failed to execute transaction"))
        } else {
            Err(JsValue::from_str("Client not initialized"))
        }
    }

    /// Executes a transaction and returns the `TransactionSummary`.
    ///
    /// If the transaction is unauthorized (auth script emits the unauthorized event),
    /// returns the summary from the error. If the transaction succeeds, constructs
    /// a summary from the executed transaction using the `auth_arg` from the transaction
    /// request as the salt (or a zero salt if not provided).
    ///
    /// # Errors
    /// - If there is an internal failure during execution.
    #[wasm_bindgen(js_name = "executeForSummary")]
    pub async fn execute_for_summary(
        &mut self,
        account_id: &AccountId,
        transaction_request: &TransactionRequest,
    ) -> Result<TransactionSummary, JsValue> {
        if let Some(client) = self.get_mut_inner() {
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
                Err(err) => Err(js_error_with_context(err, "failed to execute transaction")),
            }
        } else {
            Err(JsValue::from_str("Client not initialized"))
        }
    }

    /// Generates a transaction proof using either the provided prover or the client's default
    /// prover if none is supplied.
    #[wasm_bindgen(js_name = "proveTransaction")]
    pub async fn prove_transaction(
        &mut self,
        transaction_result: &TransactionResult,
        prover: Option<TransactionProver>,
    ) -> Result<ProvenTransaction, JsValue> {
        if let Some(client) = self.get_mut_inner() {
            let prover_arc =
                prover.map_or_else(|| client.prover(), |custom_prover| custom_prover.get_prover());

            Box::pin(client.prove_transaction_with(transaction_result.native(), prover_arc))
                .await
                .map(Into::into)
                .map_err(|err| js_error_with_context(err, "failed to prove transaction"))
        } else {
            Err(JsValue::from_str("Client not initialized"))
        }
    }

    #[wasm_bindgen(js_name = "submitProvenTransaction")]
    pub async fn submit_proven_transaction(
        &mut self,
        proven_transaction: &ProvenTransaction,
        transaction_result: &TransactionResult,
    ) -> Result<u32, JsValue> {
        if let Some(client) = self.get_mut_inner() {
            let native_proven: NativeProvenTransaction = proven_transaction.clone().into();
            client
                .submit_proven_transaction(native_proven, transaction_result.native())
                .await
                .map(|block_number| block_number.as_u32())
                .map_err(|err| js_error_with_context(err, "failed to submit proven transaction"))
        } else {
            Err(JsValue::from_str("Client not initialized"))
        }
    }

    #[wasm_bindgen(js_name = "applyTransaction")]
    pub async fn apply_transaction(
        &mut self,
        transaction_result: &TransactionResult,
        submission_height: u32,
    ) -> Result<TransactionStoreUpdate, JsValue> {
        if let Some(client) = self.get_mut_inner() {
            let update = Box::pin(client.get_transaction_store_update(
                transaction_result.native(),
                BlockNumber::from(submission_height),
            ))
            .await
            .map(TransactionStoreUpdate::from)
            .map_err(|err| js_error_with_context(err, "failed to build transaction update"))?;

            let native_update: NativeTransactionStoreUpdate = (&update).into();
            Box::pin(client.apply_transaction_update(native_update))
                .await
                .map_err(|err| js_error_with_context(err, "failed to apply transaction result"))?;

            Ok(update)
        } else {
            Err(JsValue::from_str("Client not initialized"))
        }
    }

    #[wasm_bindgen(js_name = "newMintTransactionRequest")]
    pub fn new_mint_transaction_request(
        &mut self,
        target_account_id: &AccountId,
        faucet_id: &AccountId,
        note_type: NoteType,
        amount: u64,
    ) -> Result<TransactionRequest, JsValue> {
        let fungible_asset = FungibleAsset::new(faucet_id.into(), amount)
            .map_err(|err| js_error_with_context(err, "failed to create fungible asset"))?;

        let mint_transaction_request = {
            let client = self.get_mut_inner().ok_or_else(|| {
                JsValue::from_str("Client not initialized while generating transaction request")
            })?;

            NativeTransactionRequestBuilder::new()
                .build_mint_fungible_asset(
                    fungible_asset,
                    target_account_id.into(),
                    note_type.into(),
                    client.rng(),
                )
                .map_err(|err| {
                    js_error_with_context(err, "failed to create mint transaction request")
                })?
        };

        Ok(mint_transaction_request.into())
    }

    #[wasm_bindgen(js_name = "newSendTransactionRequest")]
    pub fn new_send_transaction_request(
        &mut self,
        sender_account_id: &AccountId,
        target_account_id: &AccountId,
        faucet_id: &AccountId,
        note_type: NoteType,
        amount: u64,
        recall_height: Option<u32>,
        timelock_height: Option<u32>,
    ) -> Result<TransactionRequest, JsValue> {
        let client = self.get_mut_inner().ok_or_else(|| {
            JsValue::from_str("Client not initialized while generating transaction request")
        })?;

        let fungible_asset = FungibleAsset::new(faucet_id.into(), amount)
            .map_err(|err| js_error_with_context(err, "failed to create fungible asset"))?;

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
                js_error_with_context(err, "failed to create send transaction request")
            })?;

        Ok(send_transaction_request.into())
    }

    #[wasm_bindgen(js_name = "newConsumeTransactionRequest")]
    pub fn new_consume_transaction_request(
        &mut self,
        list_of_notes: Vec<Note>,
    ) -> Result<TransactionRequest, JsValue> {
        let consume_transaction_request = {
            let native_notes = list_of_notes
                .into_iter()
                .map(NativeNote::try_from)
                .collect::<Result<Vec<_>, _>>()
                .map_err(|err| {
                    JsValue::from_str(&format!("Failed to convert note to native note: {err}"))
                })?;

            NativeTransactionRequestBuilder::new()
                .build_consume_notes(native_notes)
                .map_err(|err| {
                    JsValue::from_str(&format!(
                        "Failed to create Consume Transaction Request: {err}"
                    ))
                })?
        };

        Ok(consume_transaction_request.into())
    }

    #[wasm_bindgen(js_name = "newSwapTransactionRequest")]
    pub fn new_swap_transaction_request(
        &mut self,
        sender_account_id: &AccountId,
        offered_asset_faucet_id: &AccountId,
        offered_asset_amount: u64,
        requested_asset_faucet_id: &AccountId,
        requested_asset_amount: u64,
        note_type: NoteType,
        payback_note_type: NoteType,
    ) -> Result<TransactionRequest, JsValue> {
        let offered_fungible_asset =
            FungibleAsset::new(offered_asset_faucet_id.into(), offered_asset_amount)
                .map_err(|err| {
                    js_error_with_context(err, "failed to create offered fungible asset")
                })?
                .into();

        let requested_fungible_asset =
            FungibleAsset::new(requested_asset_faucet_id.into(), requested_asset_amount)
                .map_err(|err| {
                    js_error_with_context(err, "failed to create requested fungible asset")
                })?
                .into();

        let swap_transaction_data = SwapTransactionData::new(
            sender_account_id.into(),
            offered_fungible_asset,
            requested_fungible_asset,
        );

        let swap_transaction_request = {
            let client = self.get_mut_inner().ok_or_else(|| {
                JsValue::from_str("Client not initialized while generating transaction request")
            })?;

            NativeTransactionRequestBuilder::new()
                .build_swap(
                    &swap_transaction_data,
                    note_type.into(),
                    payback_note_type.into(),
                    client.rng(),
                )
                .map_err(|err| {
                    js_error_with_context(err, "failed to create swap transaction request")
                })?
        };

        Ok(swap_transaction_request.into())
    }
}
