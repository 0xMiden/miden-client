use miden_client::auth::SigningInputs as NativeSigningInputs;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys::Uint8Array;

use crate::models::felt::Felt;
use crate::models::transaction_summary::TransactionSummary;
use crate::models::word::Word;
use crate::utils::{deserialize_from_uint8array, serialize_to_uint8array};

/// Enumerates the supported signing input variants.
#[wasm_bindgen]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SigningInputsType {
    /// Signing inputs derived from a transaction summary.
    TransactionSummary,
    /// Signing inputs consisting of arbitrary field elements.
    Arbitrary,
    /// Signing inputs represented by a single blind commitment word.
    Blind,
}

#[wasm_bindgen]
/// Wrapper for the data that gets hashed when producing a signature.
#[wasm_bindgen]
pub struct SigningInputs {
    inner: NativeSigningInputs,
}

#[wasm_bindgen]
impl SigningInputs {
    #[wasm_bindgen(js_name = "newTransactionSummary")]
    /// Creates signing inputs from a transaction summary.
    pub fn new_transaction_summary(summary: TransactionSummary) -> Self {
        Self {
            inner: NativeSigningInputs::TransactionSummary(Box::new(summary.into())),
        }
    }

    #[wasm_bindgen(js_name = "newArbitrary")]
    /// Creates signing inputs from arbitrary field elements.
    pub fn new_arbitrary(felts: Vec<Felt>) -> Self {
        Self {
            inner: NativeSigningInputs::Arbitrary(felts.into_iter().map(Into::into).collect()),
        }
    }

    #[wasm_bindgen(js_name = "newBlind")]
    /// Creates signing inputs from a single blind commitment word.
    pub fn new_blind(word: &Word) -> Self {
        Self {
            inner: NativeSigningInputs::Blind(word.into()),
        }
    }

    #[wasm_bindgen(js_name = "transactionSummaryPayload")]
    /// Returns the underlying transaction summary when the variant matches.
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
    /// Returns the arbitrary payload when the variant matches.
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
    /// Returns the blind commitment payload when the variant matches.
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
    /// Returns the active signing input variant.
    pub fn variant_type(&self) -> SigningInputsType {
        match &self.inner {
            NativeSigningInputs::TransactionSummary(_) => SigningInputsType::TransactionSummary,
            NativeSigningInputs::Arbitrary(_) => SigningInputsType::Arbitrary,
            NativeSigningInputs::Blind(_) => SigningInputsType::Blind,
        }
    }

    #[wasm_bindgen(js_name = "toCommitment")]
    /// Returns the commitment over the signing inputs.
    pub fn to_commitment(&self) -> Word {
        self.inner.to_commitment().into()
    }

    #[wasm_bindgen(js_name = "toElements")]
    /// Returns the signing inputs as an array of field elements.
    pub fn to_elements(&self) -> Vec<Felt> {
        self.inner.to_elements().into_iter().map(Into::into).collect()
    }

    /// Serializes the signing inputs into bytes.
    pub fn serialize(&self) -> Uint8Array {
        serialize_to_uint8array(&self.inner)
    }

    /// Deserializes signing inputs from bytes.
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
