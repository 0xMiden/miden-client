use miden_client::auth::Signature as NativeSignature;

#[cfg(feature = "napi")]
use miden_client::{Deserializable, Serializable};
#[cfg(feature = "napi")]
use napi::bindgen_prelude::*;

#[cfg(feature = "wasm")]
use wasm_bindgen_futures::js_sys::Uint8Array;

use crate::prelude::*;

use crate::models::felt::Felt;
use crate::models::word::Word;
use crate::platform::{self, JsResult};
#[cfg(feature = "wasm")]
use crate::utils::serialize_to_uint8array;

/// Cryptographic signature produced by supported auth schemes.
#[bindings]
#[derive(Clone)]
pub struct Signature(NativeSignature);

// Shared methods
#[bindings]
impl Signature {
    /// Converts the signature to the prepared field elements expected by verifying code.
    #[bindings]
    pub fn to_prepared_signature(&self, message: &Word) -> Vec<Felt> {
        self.0
            .to_prepared_signature(message.clone().into())
            .into_iter()
            .map(Into::into)
            .collect()
    }

    /// Deserializes a signature from bytes.
    #[bindings(factory)]
    pub fn deserialize(bytes: &platform::JsBytes) -> JsResult<Signature> {
        let native_signature = platform::deserialize_from_bytes::<NativeSignature>(bytes)?;
        Ok(Signature(native_signature))
    }
}

// wasm-specific methods
#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl Signature {
    /// Serializes the signature into bytes.
    pub fn serialize(&self) -> Uint8Array {
        serialize_to_uint8array(&self.0)
    }
}

// napi-specific methods
#[cfg(feature = "napi")]
#[napi_derive::napi]
impl Signature {
    /// Serializes the signature into bytes.
    #[napi]
    pub fn serialize(&self) -> Buffer {
        let bytes = self.0.to_bytes();
        Buffer::from(bytes)
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
