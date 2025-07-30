use hex::ToHex;
use miden_client::utils::{Deserializable, Serializable};
use miden_objects::{Word as NativeWord, crypto::dsa::rpo_falcon512::SecretKey as NativeSecretKey};
use rand::{SeedableRng, rngs::StdRng};
use wasm_bindgen::prelude::*;

use crate::models::{public_key::PublicKey, signature::Signature, word::Word};

#[wasm_bindgen]
pub struct SecretKey(NativeSecretKey);

#[wasm_bindgen]
impl SecretKey {
    #[wasm_bindgen(js_name = "withRng")]
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
    pub fn public_key(&self) -> PublicKey {
        self.0.public_key().into()
    }

    #[wasm_bindgen(js_name = "sign")]
    pub fn sign(&self, message: &Word) -> Result<Signature, JsValue> {
        let native_message: NativeWord = message.into();
        let mut rng = StdRng::from_os_rng();
        let signature = self.0.sign_with_rng(native_message, &mut rng);
        Ok(signature.into())
    }

    #[wasm_bindgen(js_name = "toHex")]
    pub fn to_hex(&self) -> String {
        self.0.to_bytes().encode_hex()
    }

    #[wasm_bindgen(js_name = "fromHex")]
    pub fn from_hex(hex: &str) -> Result<SecretKey, JsValue> {
        let bytes = hex::decode(&hex)
            .map_err(|err| JsValue::from_str(&format!("Invalid hex string: {err}")))?;
        let native_secret_key = NativeSecretKey::read_from_bytes(&bytes)
            .map_err(|err| JsValue::from_str(&format!("Invalid secret key string: {err}")))?;
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
