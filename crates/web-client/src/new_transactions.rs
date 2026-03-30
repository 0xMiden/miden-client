use alloc::collections::BTreeSet;

use js_export_macro::js_export;
use miden_client::ClientError;
use miden_client::asset::FungibleAsset;
use miden_client::note::{BlockNumber, Note as NativeNote};
use miden_client::transaction::{
    ForeignAccount as NativeForeignAccount,
    PaymentNoteDescription,
    ProvenTransaction as NativeProvenTransaction,
    SwapTransactionData,
    TransactionExecutorError,
    TransactionRequest as NativeTransactionRequest,
    TransactionRequestBuilder as NativeTransactionRequestBuilder,
    TransactionStoreUpdate as NativeTransactionStoreUpdate,
    TransactionSummary as NativeTransactionSummary,
};

use crate::models::NoteType;
use crate::models::account_id::AccountId;
use crate::models::advice_inputs::AdviceInputs;
use crate::models::felt::Felt;
use crate::models::miden_arrays::{FeltArray, ForeignAccountArray};
use crate::models::note::Note;
use crate::models::proven_transaction::ProvenTransaction;
use crate::models::provers::TransactionProver;
use crate::models::transaction_id::TransactionId;
use crate::models::transaction_request::TransactionRequest;
use crate::models::transaction_result::TransactionResult;
use crate::models::transaction_script::TransactionScript;
use crate::models::transaction_store_update::TransactionStoreUpdate;
use crate::models::transaction_summary::TransactionSummary;
use crate::platform::{JsErr, from_str_err, js_u64_to_u64, maybe_wrap_send};
use crate::{WebClient, js_error_with_context};

#[js_export]
impl WebClient {
    #[js_export(js_name = "newMintTransactionRequest")]
    pub async fn new_mint_transaction_request(
        &self,
        target_account_id: &AccountId,
        faucet_id: &AccountId,
        note_type: NoteType,
        amount: JsU64,
    ) -> Result<TransactionRequest, JsErr> {
        let amount = js_u64_to_u64(amount);
        let fungible_asset = FungibleAsset::new(faucet_id.into(), amount)
            .map_err(|err| js_error_with_context(err, "failed to create fungible asset"))?;

        let mint_transaction_request = {
            let mut guard = self.get_mut_inner().await;
            let client = guard.as_mut().ok_or_else(|| {
                from_str_err("Client not initialized while generating transaction request")
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

    #[js_export(js_name = "newSendTransactionRequest")]
    pub async fn new_send_transaction_request(
        &self,
        sender_account_id: &AccountId,
        target_account_id: &AccountId,
        faucet_id: &AccountId,
        note_type: NoteType,
        amount: JsU64,
        recall_height: Option<u32>,
        timelock_height: Option<u32>,
    ) -> Result<TransactionRequest, JsErr> {
        let mut guard = self.get_mut_inner().await;
        let client = guard.as_mut().ok_or_else(|| {
            from_str_err("Client not initialized while generating transaction request")
        })?;

        let amount = js_u64_to_u64(amount);
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

    #[js_export(js_name = "newSwapTransactionRequest")]
    pub async fn new_swap_transaction_request(
        &self,
        sender_account_id: &AccountId,
        offered_asset_faucet_id: &AccountId,
        offered_asset_amount: JsU64,
        requested_asset_faucet_id: &AccountId,
        requested_asset_amount: JsU64,
        note_type: NoteType,
        payback_note_type: NoteType,
    ) -> Result<TransactionRequest, JsErr> {
        let offered_asset_amount = js_u64_to_u64(offered_asset_amount);
        let offered_fungible_asset =
            FungibleAsset::new(offered_asset_faucet_id.into(), offered_asset_amount)
                .map_err(|err| {
                    js_error_with_context(err, "failed to create offered fungible asset")
                })?
                .into();

        let requested_asset_amount = js_u64_to_u64(requested_asset_amount);
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
            let mut guard = self.get_mut_inner().await;
            let client = guard.as_mut().ok_or_else(|| {
                from_str_err("Client not initialized while generating transaction request")
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

    /// Executes a transaction specified by the request against the specified account,
    /// proves it, submits it to the network, and updates the local database.
    ///
    /// Uses the prover configured for this client.
    ///
    /// If the transaction utilizes foreign account data, there is a chance that the client doesn't
    /// have the required block header in the local database. In these scenarios, a sync to
    /// the chain tip is performed, and the required block header is retrieved.
    #[js_export(js_name = "submitNewTransaction")]
    pub async fn submit_new_transaction(
        &self,
        account_id: &AccountId,
        transaction_request: &TransactionRequest,
    ) -> Result<TransactionId, JsErr> {
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
    #[js_export(js_name = "submitNewTransactionWithProver")]
    pub async fn submit_new_transaction_with_prover(
        &self,
        account_id: &AccountId,
        transaction_request: &TransactionRequest,
        prover: &TransactionProver,
    ) -> Result<TransactionId, JsErr> {
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
    #[js_export(js_name = "executeTransaction")]
    pub async fn execute_transaction(
        &self,
        account_id: &AccountId,
        transaction_request: &TransactionRequest,
    ) -> Result<TransactionResult, JsErr> {
        let mut guard = self.get_mut_inner().await;
        let client = guard.as_mut().ok_or_else(|| from_str_err("Client not initialized"))?;
        let fut =
            Box::pin(client.execute_transaction(account_id.into(), transaction_request.into()));
        maybe_wrap_send(fut)
            .await
            .map(TransactionResult::from)
            .map_err(|err| js_error_with_context(err, "failed to execute transaction"))
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
    #[js_export(js_name = "executeForSummary")]
    pub async fn execute_for_summary(
        &self,
        account_id: &AccountId,
        transaction_request: &TransactionRequest,
    ) -> Result<TransactionSummary, JsErr> {
        let mut guard = self.get_mut_inner().await;
        let client = guard.as_mut().ok_or_else(|| from_str_err("Client not initialized"))?;
        let native_request: NativeTransactionRequest = transaction_request.into();
        // auth_arg is passed to the auth procedure as the salt for the transaction summary
        // defaults to 0 if not provided.
        let salt = native_request.auth_arg().unwrap_or_default();

        let fut = Box::pin(client.execute_transaction(account_id.into(), native_request));
        let execute_result = maybe_wrap_send(fut).await;
        match execute_result {
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
            Err(ClientError::TransactionExecutorError(TransactionExecutorError::Unauthorized(
                summary,
            ))) => Ok(TransactionSummary::from(*summary)),
            Err(err) => Err(js_error_with_context(err, "failed to execute transaction")),
        }
    }

    /// Executes the provided transaction script against the specified account
    /// and returns the resulting stack output. This is a local-only "view call"
    /// that does not submit anything to the network.
    #[js_export(js_name = "executeProgram")]
    pub async fn execute_program(
        &self,
        account_id: &AccountId,
        tx_script: &TransactionScript,
        advice_inputs: &AdviceInputs,
        foreign_accounts: &ForeignAccountArray,
    ) -> Result<FeltArray, JsErr> {
        let mut guard = self.get_mut_inner().await;
        let client = guard.as_mut().ok_or_else(|| from_str_err("Client not initialized"))?;
        let foreign_accounts_set: BTreeSet<NativeForeignAccount> =
            foreign_accounts.__inner.iter().map(|a| a.clone().into()).collect();

        let result = client
            .execute_program(
                account_id.into(),
                tx_script.into(),
                advice_inputs.into(),
                foreign_accounts_set,
            )
            .await
            .map_err(|err| js_error_with_context(err, "failed to execute program"))?;

        let felt_vec: Vec<Felt> = result.iter().map(|f| Felt::from(*f)).collect();
        Ok(felt_vec.into())
    }

    /// Generates a transaction proof using either the provided prover or the client's default
    /// prover if none is supplied.
    #[js_export(js_name = "proveTransaction")]
    pub async fn prove_transaction(
        &self,
        transaction_result: &TransactionResult,
        prover: Option<TransactionProver>,
    ) -> Result<ProvenTransaction, JsErr> {
        let mut guard = self.get_mut_inner().await;
        let client = guard.as_mut().ok_or_else(|| from_str_err("Client not initialized"))?;
        let prover_arc =
            prover.map_or_else(|| client.prover(), |custom_prover| custom_prover.get_prover());

        let fut = Box::pin(client.prove_transaction_with(transaction_result.native(), prover_arc));
        maybe_wrap_send(fut)
            .await
            .map(Into::into)
            .map_err(|err| js_error_with_context(err, "failed to prove transaction"))
    }

    #[js_export(js_name = "submitProvenTransaction")]
    pub async fn submit_proven_transaction(
        &self,
        proven_transaction: &ProvenTransaction,
        transaction_result: &TransactionResult,
    ) -> Result<u32, JsErr> {
        let mut guard = self.get_mut_inner().await;
        let client = guard.as_mut().ok_or_else(|| from_str_err("Client not initialized"))?;
        let native_proven: NativeProvenTransaction = proven_transaction.clone().into();
        client
            .submit_proven_transaction(native_proven, transaction_result.native())
            .await
            .map(|block_number| block_number.as_u32())
            .map_err(|err| js_error_with_context(err, "failed to submit proven transaction"))
    }

    #[js_export(js_name = "applyTransaction")]
    pub async fn apply_transaction(
        &self,
        transaction_result: &TransactionResult,
        submission_height: u32,
    ) -> Result<TransactionStoreUpdate, JsErr> {
        let mut guard = self.get_mut_inner().await;
        let client = guard.as_mut().ok_or_else(|| from_str_err("Client not initialized"))?;
        let fut = Box::pin(client.get_transaction_store_update(
            transaction_result.native(),
            BlockNumber::from(submission_height),
        ));
        let update = maybe_wrap_send(fut)
            .await
            .map(TransactionStoreUpdate::from)
            .map_err(|err| js_error_with_context(err, "failed to build transaction update"))?;

        let native_update: NativeTransactionStoreUpdate = (&update).into();
        let fut = Box::pin(client.apply_transaction_update(native_update));
        maybe_wrap_send(fut)
            .await
            .map_err(|err| js_error_with_context(err, "failed to apply transaction result"))?;

        Ok(update)
    }

    #[js_export(js_name = "newConsumeTransactionRequest")]
    pub fn new_consume_transaction_request(
        &self,
        list_of_notes: Vec<Note>,
    ) -> Result<TransactionRequest, JsErr> {
        let consume_transaction_request = {
            let native_notes = list_of_notes
                .into_iter()
                .map(NativeNote::try_from)
                .collect::<Result<Vec<_>, _>>()
                .map_err(|err| {
                    from_str_err(&format!("Failed to convert note to native note: {err}"))
                })?;

            NativeTransactionRequestBuilder::new()
                .build_consume_notes(native_notes)
                .map_err(|err| {
                    from_str_err(&format!("Failed to create Consume Transaction Request: {err}"))
                })?
        };

        Ok(consume_transaction_request.into())
    }
}
