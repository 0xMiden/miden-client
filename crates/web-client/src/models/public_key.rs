use hex::ToHex;
use miden_client::utils::{Deserializable, Serializable};
use miden_objects::{Word as NativeWord, crypto::dsa::rpo_falcon512::PublicKey as NativePublicKey};
use wasm_bindgen::prelude::*;

use crate::models::{signature::Signature, word::Word};

#[wasm_bindgen]
#[derive(Copy, Clone)]
pub struct PublicKey(NativePublicKey);

#[wasm_bindgen]
impl PublicKey {
    #[wasm_bindgen(js_name = "toHex")]
    pub fn to_hex(&self) -> String {
        let word: NativeWord = self.0.into();
        word.to_bytes().encode_hex()
    }

    #[wasm_bindgen(js_name = "fromHex")]
    pub fn from_hex(hex: &str) -> Result<PublicKey, JsValue> {
        let bytes = hex::decode(&hex)
            .map_err(|err| JsValue::from_str(&format!("Invalid hex string: {err}")))?;
        let word = NativeWord::read_from_bytes(&bytes)
            .map_err(|err| JsValue::from_str(&format!("Invalid public key string: {err}")))?;
        Ok(PublicKey(NativePublicKey::new(word)))
    }

    #[wasm_bindgen(js_name = "verify")]
    pub fn verify(&self, message: &Word, signature: &Signature) -> bool {
        let native_signature = signature.into();
        self.0.verify(message.into(), &native_signature)
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativePublicKey> for PublicKey {
    fn from(native_public_key: NativePublicKey) -> Self {
        PublicKey(native_public_key)
    }
}

impl From<&NativePublicKey> for PublicKey {
    fn from(native_public_key: &NativePublicKey) -> Self {
        PublicKey(*native_public_key)
    }
}

impl From<PublicKey> for NativePublicKey {
    fn from(public_key: PublicKey) -> Self {
        public_key.0
    }
}

impl From<&PublicKey> for NativePublicKey {
    fn from(public_key: &PublicKey) -> Self {
        public_key.0
    }
}
