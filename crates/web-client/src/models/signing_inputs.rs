use miden_client::auth::SigningInputs as NativeSigningInputs;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys::Uint8Array;

use crate::models::felt::Felt;
use crate::models::transaction_summary::TransactionSummary;
use crate::models::word::Word;
use crate::utils::{deserialize_from_uint8array, serialize_to_uint8array};

#[wasm_bindgen]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SigningInputsVariant {
    TransactionSummary,
    Arbitrary,
    Blind,
}

#[wasm_bindgen]
pub struct SigningInputs {
    inner: NativeSigningInputs,
}

#[wasm_bindgen]
impl SigningInputs {
    #[wasm_bindgen(js_name = "newTransactionSummary")]
    pub fn new_transaction_summary(summary: TransactionSummary) -> Self {
        Self {
            inner: NativeSigningInputs::TransactionSummary(Box::new(summary.into())),
        }
    }

    #[wasm_bindgen(js_name = "newArbitrary")]
    pub fn new_arbitrary(felts: Vec<Felt>) -> Self {
        Self {
            inner: NativeSigningInputs::Arbitrary(felts.into_iter().map(Into::into).collect()),
        }
    }

    #[wasm_bindgen(js_name = "newBlind")]
    pub fn new_blind(word: &Word) -> Self {
        Self {
            inner: NativeSigningInputs::Blind(word.into()),
        }
    }

    #[wasm_bindgen(js_name = "transactionSummaryPayload")]
    pub fn transaction_summary_payload(&self) -> Result<TransactionSummary, JsValue> {
        match &self.inner {
            NativeSigningInputs::TransactionSummary(ts) => {
                Ok(TransactionSummary::from((**ts).clone()))
            },
            _ => Err(JsValue::from_str(&format!(
                "transactionSummaryPayload requires SigningInputs::TransactionSummary (found {:?})",
                self.variant_type()
            ))),
        }
    }

    #[wasm_bindgen(js_name = "arbitraryPayload")]
    pub fn arbitrary_payload(&self) -> Result<Box<[Felt]>, JsValue> {
        match &self.inner {
            NativeSigningInputs::Arbitrary(felts) => {
                Ok(felts.iter().copied().map(Felt::from).collect::<Vec<_>>().into_boxed_slice())
            },
            _ => Err(JsValue::from_str(&format!(
                "arbitraryPayload requires SigningInputs::Arbitrary (found {:?})",
                self.variant_type()
            ))),
        }
    }

    #[wasm_bindgen(js_name = "blindPayload")]
    pub fn blind_payload(&self) -> Result<Word, JsValue> {
        match &self.inner {
            NativeSigningInputs::Blind(word) => Ok(Word::from(*word)),
            _ => Err(JsValue::from_str(&format!(
                "blindPayload requires SigningInputs::Blind (found {:?})",
                self.variant_type()
            ))),
        }
    }

    #[wasm_bindgen(getter, js_name = "variantType")]
    pub fn variant_type(&self) -> SigningInputsVariant {
        match &self.inner {
            NativeSigningInputs::TransactionSummary(_) => SigningInputsVariant::TransactionSummary,
            NativeSigningInputs::Arbitrary(_) => SigningInputsVariant::Arbitrary,
            NativeSigningInputs::Blind(_) => SigningInputsVariant::Blind,
        }
    }

    #[wasm_bindgen(js_name = "toCommitment")]
    pub fn to_commitment(&self) -> Word {
        self.inner.to_commitment().into()
    }

    #[wasm_bindgen(js_name = "toElements")]
    pub fn to_elements(&self) -> Vec<Felt> {
        self.inner.to_elements().into_iter().map(Into::into).collect()
    }

    pub fn serialize(&self) -> Uint8Array {
        serialize_to_uint8array(&self.inner)
    }

    pub fn deserialize(bytes: &Uint8Array) -> Result<SigningInputs, JsValue> {
        let native_signing_inputs = deserialize_from_uint8array::<NativeSigningInputs>(bytes)?;
        Ok(SigningInputs { inner: native_signing_inputs })
    }
}
