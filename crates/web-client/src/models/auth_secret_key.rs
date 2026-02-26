use miden_client::auth::AuthSecretKey as NativeAuthSecretKey;
use miden_client::utils::Serializable;
use miden_client::{Felt as NativeFelt, Word as NativeWord};
use rand::SeedableRng;
use rand::rngs::StdRng;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys::Uint8Array;

use super::felt::Felt;
use super::public_key::PublicKey;
use super::signature::Signature;
use super::signing_inputs::SigningInputs;
use super::word::Word;
use crate::utils::{deserialize_from_uint8array, serialize_to_uint8array};

#[derive(Clone, Debug)]
#[wasm_bindgen]
pub struct AuthSecretKey(NativeAuthSecretKey);

#[wasm_bindgen]
impl AuthSecretKey {
    #[wasm_bindgen(js_name = "rpoFalconWithRNG")]
    pub fn rpo_falcon_with_rng(seed: Option<Vec<u8>>) -> Result<AuthSecretKey, JsValue> {
        let mut rng = Self::try_rng_from_seed(seed)?;
        Ok(NativeAuthSecretKey::new_falcon512_rpo_with_rng(&mut rng).into())
    }

    #[wasm_bindgen(js_name = "ecdsaWithRNG")]
    pub fn ecdsa_with_rng(seed: Option<Vec<u8>>) -> Result<AuthSecretKey, JsValue> {
        let mut rng = Self::try_rng_from_seed(seed)?;
        Ok(NativeAuthSecretKey::new_ecdsa_k256_keccak_with_rng(&mut rng).into())
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

    fn public_key_commitment(&self) -> NativeWord {
        self.0.public_key().to_commitment().into()
    }

    #[wasm_bindgen(js_name = "publicKey")]
    pub fn public_key(&self) -> PublicKey {
        self.0.public_key().into()
    }

    #[wasm_bindgen(js_name = "getPublicKeyAsWord")]
    pub fn get_public_key_as_word(&self) -> Word {
        self.public_key_commitment().into()
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

    pub fn deserialize(bytes: &Uint8Array) -> Result<AuthSecretKey, JsValue> {
        let native_secret_key = deserialize_from_uint8array::<NativeAuthSecretKey>(bytes)?;
        Ok(AuthSecretKey(native_secret_key))
    }

    #[wasm_bindgen(js_name = "getRpoFalcon512SecretKeyAsFelts")]
    pub fn get_rpo_falcon_512_secret_key_as_felts(&self) -> Result<Vec<Felt>, JsValue> {
        let secret_key_as_bytes = match &self.0 {
            NativeAuthSecretKey::Falcon512Rpo(key) => key.to_bytes(),
            _ => return Err(JsValue::from_str("Key is not an RPO Falcon 512 key")),
        };

        let secret_key_as_native_felts = secret_key_as_bytes
            .iter()
            .map(|a| NativeFelt::new(u64::from(*a)))
            .collect::<Vec<NativeFelt>>();

        Ok(secret_key_as_native_felts.into_iter().map(Into::into).collect())
    }

    /// Returns the ECDSA k256 Keccak secret key bytes encoded as felts.
    #[wasm_bindgen(js_name = "getEcdsaK256KeccakSecretKeyAsFelts")]
    pub fn get_ecdsa_k256_keccak_secret_key_as_felts(&self) -> Result<Vec<Felt>, JsValue> {
        let secret_key_as_bytes = match &self.0 {
            NativeAuthSecretKey::EcdsaK256Keccak(key) => key.to_bytes(),
            _ => return Err(JsValue::from_str("Key is not an ECDSA K256 Keccak key")),
        };

        let secret_key_as_native_felts = secret_key_as_bytes
            .iter()
            .map(|a| NativeFelt::new(u64::from(*a)))
            .collect::<Vec<NativeFelt>>();

        Ok(secret_key_as_native_felts.into_iter().map(Into::into).collect())
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeAuthSecretKey> for AuthSecretKey {
    fn from(native_auth_secret_key: NativeAuthSecretKey) -> Self {
        AuthSecretKey(native_auth_secret_key)
    }
}

impl From<&NativeAuthSecretKey> for AuthSecretKey {
    fn from(native_auth_secret_key: &NativeAuthSecretKey) -> Self {
        AuthSecretKey(native_auth_secret_key.clone())
    }
}

impl From<AuthSecretKey> for NativeAuthSecretKey {
    fn from(auth_secret_key: AuthSecretKey) -> Self {
        auth_secret_key.0
    }
}

impl From<&AuthSecretKey> for NativeAuthSecretKey {
    fn from(auth_secret_key: &AuthSecretKey) -> Self {
        auth_secret_key.0.clone()
    }
}

impl<'a> From<&'a AuthSecretKey> for &'a NativeAuthSecretKey {
    fn from(auth_secret_key: &'a AuthSecretKey) -> Self {
        &auth_secret_key.0
    }
}
