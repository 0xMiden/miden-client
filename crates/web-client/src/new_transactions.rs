use miden_client::asset::FungibleAsset;
use miden_client::note::{BlockNumber, NoteId as NativeNoteId};
use miden_client::transaction::{
    PaymentNoteDescription,
    ProvenTransaction as NativeProvenTransaction,
    SwapTransactionData,
    TransactionRequestBuilder as NativeTransactionRequestBuilder,
    TransactionStoreUpdate as NativeTransactionStoreUpdate,
};
use wasm_bindgen::prelude::*;

use crate::models::account_id::AccountId;
use crate::models::note_type::NoteType;
use crate::models::proven_transaction::ProvenTransaction;
use crate::models::provers::TransactionProver;
use crate::models::transaction_id::TransactionId;
use crate::models::transaction_pipeline::TransactionPipeline;
use crate::models::transaction_request::TransactionRequest;
use crate::models::transaction_store_update::TransactionStoreUpdate;
use crate::{WebClient, js_error_with_context};

#[wasm_bindgen]
impl WebClient {
    /// Executes a transaction specified by the request against the specified account,
    /// proves it, submits it to the network, and updates the local database.
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
        let mut pipeline = self.execute_transaction(account_id, transaction_request).await?;

        let tx_id = pipeline.id()?;

        let prover = self
            .get_mut_inner()
            .ok_or_else(|| JsValue::from_str("Client not initialized while proving transaction"))?
            .prover();

        pipeline.prove_transaction(Some(TransactionProver::from(prover))).await?;

        let transaction_update = pipeline.submit_proven_transaction().await?;

        self.apply_transaction(transaction_update).await?;

        Ok(tx_id)
    }

    /// Executes a transaction specified by the request against the specified account but does not
    /// submit it to the network nor update the local database. The returned [`TransactionPipeline`]
    /// retains all intermediate artifacts (request, execution results, proofs) needed to continue
    /// with the transaction lifecycle.
    ///
    /// If the transaction utilizes foreign account data, there is a chance that the client doesn't
    /// have the required block header in the local database. In these scenarios, a sync to
    /// the chain tip is performed, and the required block header is retrieved.
    #[wasm_bindgen(js_name = "executeTransaction")]
    pub async fn execute_transaction(
        &mut self,
        account_id: &AccountId,
        transaction_request: &TransactionRequest,
    ) -> Result<TransactionPipeline, JsValue> {
        if let Some(client) = self.get_mut_inner() {
            let pipeline =
                Box::pin(client.execute_transaction(account_id.into(), transaction_request.into()))
                    .await
                    .map_err(|err| {
                        js_error_with_context(err, "failed to execute transaction pipeline")
                    })?;

            Ok(TransactionPipeline::new(pipeline))
        } else {
            Err(JsValue::from_str("Client not initialized"))
        }
    }

    #[wasm_bindgen(js_name = "newTransactionPipeline")]
    pub fn new_transaction_pipeline(
        &mut self,
        transaction_request: &TransactionRequest,
    ) -> Result<TransactionPipeline, JsValue> {
        if let Some(client) = self.get_mut_inner() {
            let pipeline = client.new_transaction_pipeline(transaction_request.into());
            Ok(TransactionPipeline::new(pipeline))
        } else {
            Err(JsValue::from_str("Client not initialized"))
        }
    }

    #[wasm_bindgen(js_name = "submitProvenTransaction")]
    pub async fn submit_proven_transaction(
        &mut self,
        proven_transaction: &ProvenTransaction,
    ) -> Result<u32, JsValue> {
        if let Some(client) = self.get_mut_inner() {
            let native_proven: NativeProvenTransaction = proven_transaction.clone().into();
            client
                .submit_proven_transaction(native_proven)
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
        tx_update: TransactionStoreUpdate,
    ) -> Result<(), JsValue> {
        let native_transaction_result: NativeTransactionStoreUpdate = tx_update.into();

        if let Some(client) = self.get_mut_inner() {
            Box::pin(client.apply_transaction(native_transaction_result))
                .await
                .map_err(|err| js_error_with_context(err, "failed to apply transaction"))?;
            Ok(())
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
        list_of_note_ids: Vec<String>,
    ) -> Result<TransactionRequest, JsValue> {
        let consume_transaction_request = {
            let native_note_ids = list_of_note_ids
                .into_iter()
                .map(|note_id| NativeNoteId::try_from_hex(note_id.as_str()))
                .collect::<Result<Vec<_>, _>>()
                .map_err(|err| {
                    JsValue::from_str(&format!(
                        "Failed to convert note id to native note id: {err}"
                    ))
                })?;

            NativeTransactionRequestBuilder::new()
                .build_consume_notes(native_note_ids)
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
