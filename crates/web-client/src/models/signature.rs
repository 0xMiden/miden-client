use miden_client::crypto::rpo_falcon512::Signature as NativeSignature;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys::Uint8Array;

use crate::utils::{deserialize_from_uint8array, serialize_to_uint8array};

#[wasm_bindgen]
#[derive(Clone)]
pub struct Signature(NativeSignature);

#[wasm_bindgen]
impl Signature {
    pub fn serialize(&self) -> Uint8Array {
        serialize_to_uint8array(&self.0)
    }

    pub fn deserialize(bytes: &Uint8Array) -> Result<Signature, JsValue> {
        let native_signature =
            deserialize_from_uint8array::<NativeSignature>(bytes).map_err(Into::<JsValue>::into)?;
        Ok(Signature(native_signature))
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
