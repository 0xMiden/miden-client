use alloc::sync::Arc;

use miden_client::transaction::{
    TransactionPipeline as NativeTransactionPipeline,
    TransactionProver as NativeTransactionProver,
};
use miden_objects::note::BlockNumber;
use wasm_bindgen::prelude::*;

use crate::js_error_with_context;
use crate::models::executed_transaction::ExecutedTransaction;
use crate::models::proven_transaction::ProvenTransaction;
use crate::models::provers::TransactionProver;
use crate::models::transaction_id::TransactionId;
use crate::models::transaction_request::TransactionRequest;
use crate::models::transaction_request::note_details_and_tag::NoteDetailsAndTag;
use crate::models::transaction_store_update::TransactionStoreUpdate;

#[derive(Clone)]
#[wasm_bindgen]
pub struct TransactionPipeline {
    pipeline: NativeTransactionPipeline,
    default_prover: Option<Arc<dyn NativeTransactionProver + Send + Sync>>,
}

#[wasm_bindgen]
impl TransactionPipeline {
    pub fn id(&self) -> Result<TransactionId, JsValue> {
        self.pipeline.id().map(Into::into).map_err(|err| {
            js_error_with_context(err, "failed to retrieve transaction id from pipeline")
        })
    }

    #[wasm_bindgen(js_name = "transactionRequest")]
    pub fn transaction_request(&self) -> TransactionRequest {
        self.pipeline.request().into()
    }

    #[wasm_bindgen(js_name = "executedTransaction")]
    pub fn executed_transaction(&self) -> Result<ExecutedTransaction, JsValue> {
        self.pipeline.executed_transaction().map(Into::into).map_err(|err| {
            js_error_with_context(err, "pipeline has not executed a transaction yet")
        })
    }

    #[wasm_bindgen(js_name = "provenTransaction")]
    pub fn proven_transaction(&self) -> Option<ProvenTransaction> {
        self.pipeline.proven_transaction().map(Into::into)
    }

    #[wasm_bindgen(js_name = "futureNotes")]
    pub fn future_notes(&self) -> Vec<NoteDetailsAndTag> {
        self.pipeline
            .future_notes()
            .iter()
            .cloned()
            .map(|(note_details, note_tag)| {
                NoteDetailsAndTag::new(note_details.into(), note_tag.into())
            })
            .collect()
    }

    #[wasm_bindgen(js_name = "getTransactionUpdate")]
    pub fn get_transaction_update(&self) -> Result<TransactionStoreUpdate, JsValue> {
        self.pipeline
            .get_transaction_update()
            .map(Into::into)
            .map_err(|err| js_error_with_context(err, "failed to build transaction store update"))
    }

    #[wasm_bindgen(js_name = "getTransactionUpdateWithHeight")]
    pub fn get_transaction_update_with_height(
        &self,
        submission_height: u32,
    ) -> Result<TransactionStoreUpdate, JsValue> {
        self.pipeline
            .get_transaction_update_with_height(BlockNumber::from(submission_height))
            .map(Into::into)
            .map_err(|err| js_error_with_context(err, "failed to build transaction store update"))
    }

    #[wasm_bindgen(js_name = "proveTransaction")]
    pub async fn prove_transaction(
        &mut self,
        prover: Option<TransactionProver>,
    ) -> Result<ProvenTransaction, JsValue> {
        let prover_arc = match prover {
            Some(custom_prover) => custom_prover.get_prover(),
            None => self.default_prover.clone().ok_or_else(|| {
                JsValue::from_str(
                    "No prover available: provide one or start the pipeline via WebClient",
                )
            })?,
        };

        self.pipeline
            .prove_transaction(prover_arc)
            .await
            .map(Into::into)
            .map_err(|err| js_error_with_context(err, "failed to prove transaction"))
    }

    #[wasm_bindgen(js_name = "submitProvenTransaction")]
    pub async fn submit_proven_transaction(&mut self) -> Result<TransactionStoreUpdate, JsValue> {
        self.pipeline
            .submit_proven_transaction()
            .await
            .map(Into::into)
            .map_err(|err| js_error_with_context(err, "failed to submit proven transaction"))
    }
}

impl TransactionPipeline {
    pub(crate) fn new(
        pipeline: NativeTransactionPipeline,
        default_prover: Option<Arc<dyn NativeTransactionProver + Send + Sync>>,
    ) -> Self {
        Self { pipeline, default_prover }
    }

    pub(crate) fn into_inner(self) -> NativeTransactionPipeline {
        self.pipeline
    }

    pub(crate) fn default_prover(&self) -> Option<Arc<dyn NativeTransactionProver + Send + Sync>> {
        self.default_prover.clone()
    }
}
