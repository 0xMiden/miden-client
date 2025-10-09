use miden_client::auth::SigningInputs as NativeSigningInputs;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys::Uint8Array;

use crate::models::felt::Felt;
use crate::models::transaction_summary::TransactionSummary;
use crate::models::word::Word;
use crate::utils::{deserialize_from_uint8array, serialize_to_uint8array};

#[wasm_bindgen]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SigningInputsKind {
    TransactionSummary,
    Arbitrary,
    Blind,
}

#[wasm_bindgen]
pub struct SigningInputsTagged {
    kind: SigningInputsKind,
    summary: Option<TransactionSummary>,
    arbitrary: Option<Box<[Felt]>>,
    blind: Option<Word>,
}

#[wasm_bindgen]
impl SigningInputsTagged {
    #[wasm_bindgen(getter)]
    pub fn kind(&self) -> SigningInputsKind {
        self.kind
    }

    // Non-consuming getters (undefined if not that variant)
    #[wasm_bindgen(getter)]
    pub fn summary(&self) -> Option<TransactionSummary> {
        self.summary.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn arbitrary(&self) -> Option<Box<[Felt]>> {
        self.arbitrary.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn blind(&self) -> Option<Word> {
        self.blind.clone()
    }
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
        Ok(SigningInputs{inner: native_signing_inputs})
    }

    /// Borrowing/clone version
    #[wasm_bindgen(js_name = "decompose")]
    pub fn decompose(&self) -> SigningInputsTagged {
        match &self.inner {
            NativeSigningInputs::TransactionSummary(ts) => SigningInputsTagged {
                kind: SigningInputsKind::TransactionSummary,
                summary: Some(TransactionSummary::from((**ts).clone())),
                arbitrary: None,
                blind: None,
            },
            NativeSigningInputs::Arbitrary(felts) => SigningInputsTagged {
                kind: SigningInputsKind::Arbitrary,
                arbitrary: Some(
                    felts.iter().cloned().map(Felt::from).collect::<Vec<_>>().into_boxed_slice()
                ),
                summary: None,
                blind: None,
            },
            NativeSigningInputs::Blind(word) => SigningInputsTagged {
                kind: SigningInputsKind::Blind,
                blind: Some(Word::from(*word)),
                summary: None,
                arbitrary: None,
            },
        }
    }
}
