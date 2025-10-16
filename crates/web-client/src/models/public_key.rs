use miden_client::auth::Signature as NativeSignature;
use miden_client::crypto::rpo_falcon512::PublicKey as NativePublicKey;
use miden_client::{Deserializable, Word as NativeWord};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys::Uint8Array;

use crate::js_error_with_context;
use crate::models::signature::Signature;
use crate::models::signing_inputs::SigningInputs;
use crate::models::word::Word;
use crate::utils::serialize_to_uint8array;

#[wasm_bindgen]
#[derive(Clone)]
pub struct PublicKey(NativePublicKey);

#[wasm_bindgen]
impl PublicKey {
    pub fn serialize(&self) -> Uint8Array {
        let native_public_key = &self.0;
        serialize_to_uint8array(&native_public_key)
    }

    pub fn deserialize(bytes: &Uint8Array) -> Result<PublicKey, JsValue> {
        let native_public_key = NativePublicKey::read_from_bytes(&bytes.to_vec())
            .map_err(|e| js_error_with_context(e, "Failed to deserialize public key"))?;
        Ok(PublicKey(native_public_key))
    }

    pub fn verify(&self, message: &Word, signature: &Signature) -> bool {
        self.verify_data(&SigningInputs::new_blind(message), signature)
    }

    #[wasm_bindgen(js_name = "verifyData")]
    pub fn verify_data(&self, signing_inputs: &SigningInputs, signature: &Signature) -> bool {
        let native_public_key: NativePublicKey = self.into();
        let native_signature = signature.into();
        match native_signature {
            NativeSignature::RpoFalcon512(falcon_signature) => {
                let message = signing_inputs.to_commitment().into();
                native_public_key.verify(message, &falcon_signature)
            },
        }
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
        PublicKey(native_public_key.clone())
    }
}

impl From<PublicKey> for NativePublicKey {
    fn from(public_key: PublicKey) -> Self {
        public_key.0
    }
}

impl From<&PublicKey> for NativePublicKey {
    fn from(public_key: &PublicKey) -> Self {
        public_key.0.clone()
    }
}
