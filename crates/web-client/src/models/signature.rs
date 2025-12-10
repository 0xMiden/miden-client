use miden_client::auth::Signature as NativeSignature;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys::Uint8Array;

use crate::models::felt::Felt;
use crate::models::word::Word;
use crate::utils::{deserialize_from_uint8array, serialize_to_uint8array};

/// Cryptographic signature produced by supported auth schemes.
#[wasm_bindgen]
#[derive(Clone)]
pub struct Signature(NativeSignature);

#[wasm_bindgen]
impl Signature {
    /// Serializes the signature into bytes.
    pub fn serialize(&self) -> Uint8Array {
        serialize_to_uint8array(&self.0)
    }

    /// Deserializes a signature from bytes.
    pub fn deserialize(bytes: &Uint8Array) -> Result<Signature, JsValue> {
        let native_signature = deserialize_from_uint8array::<NativeSignature>(bytes)?;
        Ok(Signature(native_signature))
    }

    /// Converts the signature to the prepared field elements expected by verifying code.
    #[wasm_bindgen(js_name = "toPreparedSignature")]
    pub fn to_prepared_signature(&self, message: Word) -> Vec<Felt> {
        self.0
            .to_prepared_signature(message.into())
            .into_iter()
            .map(Into::into)
            .collect()
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeSignature> for Signature {
    fn from(native_signature: NativeSignature) -> Self {
        Signature(native_signature)
    }
}

impl From<&NativeSignature> for Signature {
    fn from(native_signature: &NativeSignature) -> Self {
        Signature(native_signature.clone())
    }
}

impl From<Signature> for NativeSignature {
    fn from(signature: Signature) -> Self {
        signature.0
    }
}

impl From<&Signature> for NativeSignature {
    fn from(signature: &Signature) -> Self {
        signature.0.clone()
    }
}
