use miden_client::auth::{PublicKey as NativePublicKey, Signature as NativeSignature};
use miden_client::crypto::rpo_falcon512::PublicKey as NativeFalconPublicKey;
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
        serialize_to_uint8array(&self.0)
    }

    pub fn deserialize(bytes: &Uint8Array) -> Result<PublicKey, JsValue> {
        let native_public_key = NativePublicKey::read_from_bytes(&bytes.to_vec())
            .map_err(|e| js_error_with_context(e, "Failed to deserialize public key"))?;
        Ok(PublicKey(native_public_key))
    }

    pub fn verify(&self, message: &Word, signature: &Signature) -> bool {
        self.verify_data(&SigningInputs::new_blind(message), signature)
    }

    #[wasm_bindgen(js_name = "toCommitment")]
    pub fn to_commitment(&self) -> Word {
        let commitment = self.0.to_commitment();
        let native_word: NativeWord = commitment.into();
        native_word.into()
    }

    #[wasm_bindgen(js_name = "recoverFrom")]
    pub fn recover_from(message: &Word, signature: &Signature) -> Result<PublicKey, JsValue> {
        let native_message: NativeWord = message.into();
        let native_signature: NativeSignature = signature.into();

        match native_signature {
            NativeSignature::RpoFalcon512(falcon_signature) => {
                let public_key =
                    NativeFalconPublicKey::recover_from(native_message, &falcon_signature);
                Ok(NativePublicKey::RpoFalcon512(public_key).into())
            },
            NativeSignature::EcdsaK256Keccak(_) => Err(JsValue::from_str(
                "recovering a public key from an EcdsaK256Keccak signature is not supported yet",
            )),
        }
    }

    #[wasm_bindgen(js_name = "verifyData")]
    pub fn verify_data(&self, signing_inputs: &SigningInputs, signature: &Signature) -> bool {
        let native_public_key: NativePublicKey = self.into();
        let message: NativeWord = signing_inputs.to_commitment().into();
        let native_signature: NativeSignature = signature.into();
        native_public_key.verify(message, native_signature)
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
