use miden_client::auth::AuthSecretKey as NativeSecretKey;
use rand::SeedableRng;
use rand::rngs::StdRng;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys::Uint8Array;

use crate::models::public_key::PublicKey;
use crate::models::signature::Signature;
use crate::models::signing_inputs::SigningInputs;
use crate::models::word::Word;
use crate::utils::{deserialize_from_uint8array, serialize_to_uint8array};

#[wasm_bindgen]
#[derive(Clone)]
pub struct SecretKey(NativeSecretKey);

#[wasm_bindgen]
impl SecretKey {
    #[wasm_bindgen(js_name = "rpoFalconWithRNG")]
    pub fn rpo_falcon_with_rng(seed: Option<Vec<u8>>) -> Result<SecretKey, JsValue> {
        let mut rng = Self::try_rng_from_seed(seed)?;
        Ok(NativeSecretKey::new_rpo_falcon512_with_rng(&mut rng).into())
    }

    #[wasm_bindgen(js_name = "ecdsaWithRNG")]
    pub fn ecdsa_with_rng(seed: Option<Vec<u8>>) -> Result<SecretKey, JsValue> {
        let mut rng = Self::try_rng_from_seed(seed)?;
        Ok(NativeSecretKey::new_ecdsa_k256_keccak_with_rng(&mut rng).into())
    }

    fn try_rng_from_seed(seed: Option<Vec<u8>>) -> Result<StdRng, JsValue> {
        match seed {
            Some(seed_bytes) => {
                // Attempt to convert the seed slice into a 32-byte array.
                let seed_array: [u8; 32] = seed_bytes
                    .try_into()
                    .map_err(|_| JsValue::from_str("Seed must be exactly 32 bytes"))?;
                Ok(StdRng::from_seed(seed_array))
            },
            None => Ok(StdRng::from_os_rng()),
        }
    }

    #[wasm_bindgen(js_name = "publicKey")]
    pub fn public_key(&self) -> PublicKey {
        match &self.0 {
            NativeSecretKey::RpoFalcon512(secret_key) => secret_key.public_key().into(),
            NativeSecretKey::EcdsaK256Keccak(_) => {
                todo!("ECDSA public keys are not supported yet")
            },
            _ => todo!("variant not yet supported"),
        }
    }

    pub fn sign(&self, message: &Word) -> Signature {
        self.sign_data(&SigningInputs::new_blind(message))
    }

    #[wasm_bindgen(js_name = "signData")]
    pub fn sign_data(&self, signing_inputs: &SigningInputs) -> Signature {
        let native_word = signing_inputs.to_commitment().into();
        (self.0.sign(native_word)).into()
    }

    pub fn serialize(&self) -> Uint8Array {
        serialize_to_uint8array(&self.0)
    }

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

impl<'a> From<&'a SecretKey> for &'a NativeSecretKey {
    fn from(secret_key: &'a SecretKey) -> Self {
        &secret_key.0
    }
}
