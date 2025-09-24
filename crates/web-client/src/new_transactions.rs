use miden_client::note::BlockNumber;
use miden_client::transaction::{
    PaymentNoteDescription,
    SwapTransactionData,
    TransactionRequestBuilder as NativeTransactionRequestBuilder,
    TransactionStoreUpdate as NativeTransactionStoreUpdate,
};
use miden_objects::asset::FungibleAsset;
use miden_objects::note::NoteId as NativeNoteId;
use wasm_bindgen::prelude::*;

use crate::models::account_id::AccountId;
use crate::models::note_type::NoteType;
use crate::models::provers::TransactionProver;
use crate::models::transaction_request::TransactionRequest;
use crate::models::transaction_store_update::TransactionStoreUpdate;
use crate::{WebClient, js_error_with_context};

#[wasm_bindgen]
impl WebClient {
    #[wasm_bindgen(js_name = "newTransaction")]
    pub async fn new_transaction(
        &mut self,
        account_id: &AccountId,
        transaction_request: &TransactionRequest,
    ) -> Result<TransactionStoreUpdate, JsValue> {
        if let Some(client) = self.get_mut_inner() {
            let transaction_pipeline =
                Box::pin(client.execute_transaction(account_id.into(), transaction_request.into()))
                    .await
                    .map_err(|err| {
                        js_error_with_context(err, "failed to create new transaction")
                    })?;
            let current_height = client
                .get_sync_height()
                .await
                .map_err(|err| js_error_with_context(err, "failed to get sync height"))?;

            let transaction_update = transaction_pipeline
                .get_transaction_update_with_height(current_height)
                .map_err(|err| js_error_with_context(err, "failed to build transaction update"))?;

            Ok(transaction_update.into())
        } else {
            Err(JsValue::from_str("Client not initialized"))
        }
    }

    #[wasm_bindgen(js_name = "submitTransaction")]
    pub async fn submit_transaction(
        &mut self,
        transaction_update: TransactionStoreUpdate,
        prover: Option<TransactionProver>,
    ) -> Result<(), JsValue> {
        let native_transaction_update: NativeTransactionStoreUpdate = transaction_update.into();
        if let Some(client) = self.get_mut_inner() {
            let prover = prover.map(|p| p.get_prover()).unwrap_or(client.prover());
            let witness = native_transaction_update.executed_transaction().clone().into();
            let proven_tx = prover.prove(witness).await.map_err(|err| {
                js_error_with_context(err, "failed to prove transaction before submission")
            })?;

            client.submit_proven_transaction(proven_tx).await.map_err(|err| {
                js_error_with_context(err, "failed to submit transaction to the network")
            })?;

            Box::pin(client.apply_transaction(native_transaction_update))
                .await
                .map_err(|err| js_error_with_context(err, "failed to apply transaction"))?;

            Ok(())
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
