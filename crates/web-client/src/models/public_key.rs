use miden_client::auth::{PublicKey as NativePublicKey, Signature as NativeSignature};
#[cfg(feature = "napi")]
use miden_client::Serializable;
use miden_client::Word as NativeWord;
#[cfg(feature = "napi")]
use napi::bindgen_prelude::*;
#[cfg(feature = "wasm")]
use wasm_bindgen_futures::js_sys::Uint8Array;

use crate::prelude::*;

use crate::models::signature::Signature;
use crate::models::signing_inputs::SigningInputs;
use crate::models::word::Word;
use crate::platform::{self, JsResult};
#[cfg(feature = "wasm")]
use crate::utils::serialize_to_uint8array;

// serialize differs in return types (Uint8Array vs Buffer) so it needs separate impl blocks.
// deserialize and recover_from are now unified in the shared block using platform::JsBytes.

#[bindings]
#[derive(Clone)]
pub struct PublicKey(pub(crate) NativePublicKey);

// Shared methods with identical implementations across both platforms.
#[bindings]
impl PublicKey {
    /// Verifies a blind message word against the signature.
    pub fn verify(&self, message: &Word, signature: &Signature) -> bool {
        self.verify_data(&SigningInputs::new_blind(message), signature)
    }

    /// Returns the commitment corresponding to this public key.
    #[bindings]
    pub fn to_commitment(&self) -> Word {
        let commitment = self.0.to_commitment();
        let native_word: NativeWord = commitment.into();
        native_word.into()
    }

    /// Verifies a signature over arbitrary signing inputs.
    #[bindings]
    pub fn verify_data(&self, signing_inputs: &SigningInputs, signature: &Signature) -> bool {
        let native_public_key: NativePublicKey = self.into();
        let message = signing_inputs.to_commitment().into();
        let native_signature: NativeSignature = signature.clone().into();
        native_public_key.verify(message, native_signature)
    }

    /// Deserializes a public key from bytes.
    #[bindings(factory)]
    pub fn deserialize(bytes: &platform::JsBytes) -> JsResult<PublicKey> {
        let native_public_key = platform::deserialize_from_bytes(bytes)?;
        Ok(PublicKey(native_public_key))
    }

    /// Recovers a public key from a signature (only supported for `RpoFalcon512`).
    #[bindings(js_name = "recoverFrom", factory)]
    pub fn recover_from(message: &Word, signature: &Signature) -> JsResult<PublicKey> {
        let native_message: NativeWord = message.into();
        let native_signature: NativeSignature = signature.into();

        match native_signature {
            NativeSignature::Falcon512Rpo(falcon_signature) => {
                let public_key = miden_client::crypto::rpo_falcon512::PublicKey::recover_from(
                    native_message,
                    &falcon_signature,
                );
                Ok(NativePublicKey::Falcon512Rpo(public_key).into())
            },
            NativeSignature::EcdsaK256Keccak(_) => Err(platform::error_from_string(
                "Recovering a public key from an EcdsaK256Keccak signature is not supported yet",
            )),
        }
    }
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl PublicKey {
    /// Serializes the public key into bytes.
    pub fn serialize(&self) -> Uint8Array {
        serialize_to_uint8array(&self.0)
    }
}

#[cfg(feature = "napi")]
#[napi_derive::napi]
impl PublicKey {
    /// Serializes the public key into bytes.
    #[napi]
    pub fn serialize(&self) -> Buffer {
        let bytes = self.0.to_bytes();
        Buffer::from(bytes)
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
