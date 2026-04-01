use js_export_macro::js_export;
use miden_client::auth::SigningInputs as NativeSigningInputs;

use super::miden_arrays::FeltArray;
use crate::models::felt::Felt;
use crate::models::transaction_summary::TransactionSummary;
use crate::models::word::Word;
use crate::platform::{JsBytes, JsErr, from_str_err};
use crate::utils::{deserialize_from_bytes, serialize_to_bytes};

#[js_export]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SigningInputsType {
    /// Signing commitment over a transaction summary.
    TransactionSummary,
    /// Arbitrary field elements supplied by caller.
    Arbitrary,
    /// Blind commitment derived from a single word.
    Blind,
}

#[js_export]
#[derive(Clone, Debug)]
pub struct SigningInputs {
    inner: NativeSigningInputs,
}

#[js_export]
impl SigningInputs {
    /// Creates signing inputs from a transaction summary.
    #[js_export(js_name = "newTransactionSummary")]
    pub fn new_transaction_summary(summary: TransactionSummary) -> Self {
        Self {
            inner: NativeSigningInputs::TransactionSummary(Box::new(summary.into())),
        }
    }

    /// Creates signing inputs from arbitrary field elements.
    #[js_export(js_name = "newArbitrary")]
    pub fn new_arbitrary(felts: Vec<Felt>) -> Self {
        Self {
            inner: NativeSigningInputs::Arbitrary(felts.into_iter().map(Into::into).collect()),
        }
    }

    /// Creates blind signing inputs from a single word.
    #[js_export(js_name = "newBlind")]
    pub fn new_blind(word: &Word) -> Self {
        Self {
            inner: NativeSigningInputs::Blind(word.into()),
        }
    }

    /// Returns the transaction summary payload if this variant contains one.
    #[js_export(js_name = "transactionSummaryPayload")]
    pub fn transaction_summary_payload(&self) -> Result<TransactionSummary, JsErr> {
        match &self.inner {
            NativeSigningInputs::TransactionSummary(ts) => {
                Ok(TransactionSummary::from((**ts).clone()))
            },
            _ => Err(from_str_err(&format!(
                "TransactionSummaryPayload requires SigningInputs::TransactionSummary (found {:?})",
                self.variant_type()
            ))),
        }
    }

    /// Returns the arbitrary payload as an array of felts.
    #[js_export(js_name = "arbitraryPayload")]
    pub fn arbitrary_payload(&self) -> Result<FeltArray, JsErr> {
        match &self.inner {
            NativeSigningInputs::Arbitrary(felts) => {
                Ok(felts.iter().copied().map(Felt::from).collect::<Vec<_>>().into())
            },
            _ => Err(from_str_err(&format!(
                "ArbitraryPayload requires SigningInputs::Arbitrary (found {:?})",
                self.variant_type()
            ))),
        }
    }

    /// Returns the blind payload as a word.
    #[js_export(js_name = "blindPayload")]
    pub fn blind_payload(&self) -> Result<Word, JsErr> {
        match &self.inner {
            NativeSigningInputs::Blind(word) => Ok(Word::from(*word)),
            _ => Err(from_str_err(&format!(
                "BlindPayload requires SigningInputs::Blind (found {:?})",
                self.variant_type()
            ))),
        }
    }

    /// Returns which variant these signing inputs represent.
    #[js_export(getter, js_name = "variantType")]
    pub fn variant_type(&self) -> SigningInputsType {
        match &self.inner {
            NativeSigningInputs::TransactionSummary(_) => SigningInputsType::TransactionSummary,
            NativeSigningInputs::Arbitrary(_) => SigningInputsType::Arbitrary,
            NativeSigningInputs::Blind(_) => SigningInputsType::Blind,
        }
    }

    /// Returns the commitment to these signing inputs.
    #[js_export(js_name = "toCommitment")]
    pub fn to_commitment(&self) -> Word {
        self.inner.to_commitment().into()
    }

    /// Returns the inputs as field elements.
    #[js_export(js_name = "toElements")]
    pub fn to_elements(&self) -> FeltArray {
        self.inner.to_elements().into_iter().map(Into::into).collect::<Vec<_>>().into()
    }

    /// Serializes the signing inputs into bytes.
    pub fn serialize(&self) -> JsBytes {
        serialize_to_bytes(&self.inner)
    }

    /// Deserializes signing inputs from bytes.
    pub fn deserialize(bytes: JsBytes) -> Result<SigningInputs, JsErr> {
        let native_signing_inputs = deserialize_from_bytes::<NativeSigningInputs>(&bytes)?;
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

impl_napi_from_value!(SigningInputs);
