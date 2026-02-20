use miden_client::auth::SigningInputs as NativeSigningInputs;

use crate::prelude::*;
use crate::models::felt::Felt;
use crate::models::word::Word;

#[cfg(feature = "wasm")]
mod wasm_def {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    pub enum SigningInputsType {
        /// Signing commitment over a transaction summary.
        TransactionSummary,
        /// Arbitrary field elements supplied by caller.
        Arbitrary,
        /// Blind commitment derived from a single word.
        Blind,
    }
}

#[cfg(feature = "napi")]
mod napi_def {
    use napi_derive::napi;

    #[napi(string_enum)]
    pub enum SigningInputsType {
        /// Signing commitment over a transaction summary.
        TransactionSummary,
        /// Arbitrary field elements supplied by caller.
        Arbitrary,
        /// Blind commitment derived from a single word.
        Blind,
    }
}

#[cfg(feature = "wasm")]
pub use wasm_def::SigningInputsType;
#[cfg(feature = "napi")]
pub use napi_def::SigningInputsType;

#[bindings]
#[derive(Clone, Debug)]
pub struct SigningInputs {
    inner: NativeSigningInputs,
}

// Shared methods (identical signatures and bodies)
#[bindings]
impl SigningInputs {
    /// Creates blind signing inputs from a single word.
    #[bindings(js_name = "newBlind", factory)]
    pub fn new_blind(word: &Word) -> Self {
        Self {
            inner: NativeSigningInputs::Blind(word.into()),
        }
    }

    /// Creates signing inputs from arbitrary field elements.
    #[bindings(js_name = "newArbitrary", factory)]
    pub fn new_arbitrary(felts: Vec<Felt>) -> Self {
        Self {
            inner: NativeSigningInputs::Arbitrary(felts.into_iter().map(Into::into).collect()),
        }
    }

    /// Returns which variant these signing inputs represent.
    #[bindings(getter)]
    pub fn variant_type(&self) -> SigningInputsType {
        match &self.inner {
            NativeSigningInputs::TransactionSummary(_) => SigningInputsType::TransactionSummary,
            NativeSigningInputs::Arbitrary(_) => SigningInputsType::Arbitrary,
            NativeSigningInputs::Blind(_) => SigningInputsType::Blind,
        }
    }

    /// Returns the commitment to these signing inputs.
    #[bindings]
    pub fn to_commitment(&self) -> Word {
        self.inner.to_commitment().into()
    }

    /// Returns the blind payload as a word.
    #[bindings]
    pub fn blind_payload(&self) -> JsResult<Word> {
        match &self.inner {
            NativeSigningInputs::Blind(word) => Ok(Word::from(*word)),
            _ => Err(platform::error_from_string(&format!(
                "BlindPayload requires SigningInputs::Blind (found {:?})",
                self.variant_type()
            ))),
        }
    }

    /// Serializes the signing inputs into bytes.
    pub fn serialize(&self) -> platform::JsBytes {
        platform::serialize_to_bytes(&self.inner)
    }

    /// Deserializes signing inputs from bytes.
    #[bindings(factory)]
    pub fn deserialize(bytes: platform::JsBytes) -> JsResult<SigningInputs> {
        let native_signing_inputs = platform::deserialize_from_bytes::<NativeSigningInputs>(&bytes)?;
        Ok(native_signing_inputs.into())
    }
}

// wasm-specific methods
#[cfg(feature = "wasm")]
use crate::models::miden_arrays::FeltArray;
#[cfg(feature = "wasm")]
use crate::models::transaction_summary::TransactionSummary;

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl SigningInputs {
    /// Creates signing inputs from a transaction summary.
    
    pub fn new_transaction_summary(summary: TransactionSummary) -> Self {
        Self {
            inner: NativeSigningInputs::TransactionSummary(Box::new(summary.into())),
        }
    }

    /// Returns the transaction summary payload if this variant contains one.
    
    pub fn transaction_summary_payload(&self) -> JsResult<TransactionSummary> {
        match &self.inner {
            NativeSigningInputs::TransactionSummary(ts) => {
                Ok(TransactionSummary::from((**ts).clone()))
            },
            _ => Err(platform::error_from_string(&format!(
                "TransactionSummaryPayload requires SigningInputs::TransactionSummary (found {:?})",
                self.variant_type()
            ))),
        }
    }

    /// Returns the arbitrary payload as an array of felts.
    
    pub fn arbitrary_payload(&self) -> JsResult<FeltArray> {
        match &self.inner {
            NativeSigningInputs::Arbitrary(felts) => {
                Ok(felts.iter().copied().map(Felt::from).collect::<Vec<_>>().into())
            },
            _ => Err(platform::error_from_string(&format!(
                "ArbitraryPayload requires SigningInputs::Arbitrary (found {:?})",
                self.variant_type()
            ))),
        }
    }

    /// Returns the inputs as field elements.
    
    pub fn to_elements(&self) -> FeltArray {
        self.inner.to_elements().into_iter().map(Into::into).collect::<Vec<_>>().into()
    }
}

// napi-specific methods
#[cfg(feature = "napi")]
#[napi_derive::napi]
impl SigningInputs {
    /// Returns the arbitrary payload as an array of felts.
    pub fn arbitrary_payload(&self) -> JsResult<Vec<Felt>> {
        match &self.inner {
            NativeSigningInputs::Arbitrary(felts) => {
                Ok(felts.iter().copied().map(Felt::from).collect())
            },
            _ => Err(platform::error_from_string(
                "ArbitraryPayload requires SigningInputs::Arbitrary",
            )),
        }
    }

    /// Returns the inputs as field elements.
    pub fn to_elements(&self) -> Vec<Felt> {
        self.inner.to_elements().into_iter().map(Into::into).collect()
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
