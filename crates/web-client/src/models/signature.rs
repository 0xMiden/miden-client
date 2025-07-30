use hex::ToHex;
use miden_client::utils::{Deserializable, Serializable};
use miden_objects::crypto::dsa::rpo_falcon512::Signature as NativeSignature;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Clone)]
pub struct Signature(NativeSignature);

#[wasm_bindgen]
impl Signature {
    #[wasm_bindgen(js_name = "toHex")]
    pub fn to_hex(&self) -> String {
        self.0.to_bytes().encode_hex()
    }

    #[wasm_bindgen(js_name = "fromHex")]
    pub fn from_hex(hex: &str) -> Result<Signature, JsValue> {
        let bytes = hex::decode(&hex)
            .map_err(|err| JsValue::from_str(&format!("Invalid hex string: {err}")))?;
        let native_signature = NativeSignature::read_from_bytes(&bytes)
            .map_err(|err| JsValue::from_str(&format!("Invalid signature string: {err}")))?;
        Ok(Signature(native_signature))
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeSignature> for Signature {
    fn from(native_signature: NativeSignature) -> Self {
        Signature(native_signature.clone())
    }
}

impl From<&NativeSignature> for Signature {
    fn from(native_signature: &NativeSignature) -> Self {
        Signature(native_signature.clone())
    }
}

impl From<Signature> for NativeSignature {
    fn from(signature: Signature) -> Self {
        signature.0.clone()
    }
}

impl From<&Signature> for NativeSignature {
    fn from(signature: &Signature) -> Self {
        signature.0.clone()
    }
}
