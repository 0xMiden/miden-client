use miden_client::Word as NativeWord;
use miden_client::auth::Signature as NativeSignature;
use miden_client::crypto::rpo_falcon512::PublicKey as NativePublicKey;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys::Uint8Array;

use crate::models::signature::Signature;
use crate::models::signing_inputs::SigningInputs;
use crate::models::word::Word;
use crate::utils::{deserialize_from_uint8array, serialize_to_uint8array};

#[wasm_bindgen]
#[derive(Copy, Clone)]
pub struct PublicKey(NativePublicKey);

#[wasm_bindgen]
impl PublicKey {
    pub fn serialize(&self) -> Uint8Array {
        let native_word: NativeWord = self.0.into();
        serialize_to_uint8array(&native_word)
    }

    pub fn deserialize(bytes: &Uint8Array) -> Result<PublicKey, JsValue> {
        let native_word = deserialize_from_uint8array::<NativeWord>(bytes)?;
        let native_public_key = NativePublicKey::new(native_word);
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

impl From<NativeWord> for PublicKey {
    fn from(word: NativeWord) -> Self {
        PublicKey(NativePublicKey::new(word))
    }
}
