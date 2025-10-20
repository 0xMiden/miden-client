use miden_client::auth::SigningInputs as NativeSigningInputs;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys::Uint8Array;

use crate::models::felt::Felt;
use crate::models::transaction_summary::TransactionSummary;
use crate::models::word::Word;
use crate::utils::{deserialize_from_uint8array, serialize_to_uint8array};

#[wasm_bindgen]
#[derive(Clone)]
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

    #[wasm_bindgen(getter, js_name = "variantType")]
    pub fn variant_type(&self) -> String {
        match &self.inner {
            NativeSigningInputs::TransactionSummary(_) => "TransactionSummary".to_string(),
            NativeSigningInputs::Arbitrary(_) => "Arbitrary".to_string(),
            NativeSigningInputs::Blind(_) => "Blind".to_string(),
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
        Ok(native_signing_inputs.into())
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeSigningInputs> for SigningInputs {
    fn from(native_signing_inputs: NativeSigningInputs) -> Self {
        SigningInputs { inner: native_signing_inputs }
    }
}

impl From<&NativeSigningInputs> for SigningInputs {
    fn from(native_signing_inputs: &NativeSigningInputs) -> Self {
        SigningInputs { inner: native_signing_inputs.clone() }
    }
}

impl From<SigningInputs> for NativeSigningInputs {
    fn from(signing_inputs: SigningInputs) -> Self {
        signing_inputs.inner
    }
}

impl From<&SigningInputs> for NativeSigningInputs {
    fn from(signing_inputs: &SigningInputs) -> Self {
        signing_inputs.inner.clone()
    }
}
