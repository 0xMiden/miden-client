use miden_client::auth::Signature as NativeSignature;
use miden_client::crypto::rpo_falcon512::SecretKey as NativeSecretKey;
use rand::SeedableRng;
use rand::rngs::StdRng;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys::Uint8Array;

use crate::models::public_key::PublicKey;
use crate::models::signature::Signature;
use crate::models::signing_inputs::SigningInputs;
use crate::models::word::Word;
use crate::utils::{deserialize_from_uint8array, serialize_to_uint8array};

/// Secret key capable of producing RPO Falcon signatures.
#[wasm_bindgen]
pub struct SecretKey(NativeSecretKey);

#[wasm_bindgen]
impl SecretKey {
    #[wasm_bindgen(js_name = "withRng")]
    /// Generates a new secret key using an optional RNG seed.
    pub fn with_rng(seed: Option<Vec<u8>>) -> Result<SecretKey, JsValue> {
        let mut rng = match seed {
            Some(seed_bytes) => {
                // Attempt to convert the seed slice into a 32-byte array.
                let seed_array: [u8; 32] = seed_bytes
                    .try_into()
                    .map_err(|_| JsValue::from_str("Seed must be exactly 32 bytes"))?;
                StdRng::from_seed(seed_array)
            },
            None => StdRng::from_os_rng(),
        };
        Ok(SecretKey(NativeSecretKey::with_rng(&mut rng)))
    }

    #[wasm_bindgen(js_name = "publicKey")]
    /// Returns the public key corresponding to this secret key.
    pub fn public_key(&self) -> PublicKey {
        self.0.public_key().into()
    }

    // TODO: update to sign instead of sign_with_rng once miden-objects uses miden-crypto 0.16
    /// Signs a simple message commitment and returns the signature.
    pub fn sign(&self, message: &Word) -> Signature {
        self.sign_data(&SigningInputs::new_blind(message))
    }

    // TODO: update to sign instead of sign_with_rng once miden-objects uses miden-crypto 0.16
    #[wasm_bindgen(js_name = "signData")]
    /// Signs the provided signing inputs and returns the resulting signature.
    pub fn sign_data(&self, signing_inputs: &SigningInputs) -> Signature {
        let mut rng = StdRng::from_os_rng();
        let native_word = signing_inputs.to_commitment().into();
        NativeSignature::from(self.0.sign_with_rng(native_word, &mut rng)).into()
    }

    /// Serializes the secret key into bytes.
    pub fn serialize(&self) -> Uint8Array {
        serialize_to_uint8array(&self.0)
    }

    /// Deserializes a secret key from bytes.
    pub fn deserialize(bytes: &Uint8Array) -> Result<SecretKey, JsValue> {
        let native_secret_key = deserialize_from_uint8array::<NativeSecretKey>(bytes)?;
        Ok(SecretKey(native_secret_key))
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeSecretKey> for SecretKey {
    fn from(native_secret_key: NativeSecretKey) -> Self {
        SecretKey(native_secret_key)
    }
}

impl From<&NativeSecretKey> for SecretKey {
    fn from(native_secret_key: &NativeSecretKey) -> Self {
        SecretKey(native_secret_key.clone())
    }
}

impl From<SecretKey> for NativeSecretKey {
    fn from(secret_key: SecretKey) -> Self {
        secret_key.0
    }
}

impl From<&SecretKey> for NativeSecretKey {
    fn from(secret_key: &SecretKey) -> Self {
        secret_key.0.clone()
    }
}
