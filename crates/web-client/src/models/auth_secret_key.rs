use miden_client::auth::AuthSecretKey as NativeAuthSecretKey;
use miden_client::utils::Serializable;
use miden_client::{Felt as NativeFelt, Word as NativeWord};
use rand::SeedableRng;
use rand::rngs::StdRng;

#[cfg(feature = "napi")]
use miden_client::Deserializable;
#[cfg(feature = "napi")]
use napi::bindgen_prelude::*;

#[cfg(feature = "wasm")]
use wasm_bindgen_futures::js_sys::Uint8Array;

use super::felt::Felt;
use super::public_key::PublicKey;
use super::signature::Signature;
use super::signing_inputs::SigningInputs;
use super::word::Word;
use crate::platform;
use crate::prelude::*;
#[cfg(feature = "wasm")]
use crate::utils::{deserialize_from_uint8array, serialize_to_uint8array};

#[bindings]
#[derive(Clone, Debug)]
pub struct AuthSecretKey(NativeAuthSecretKey);

// Shared methods (identical signatures and bodies)
#[bindings]
impl AuthSecretKey {
    fn public_key_commitment(&self) -> NativeWord {
        match &self.0 {
            NativeAuthSecretKey::Falcon512Rpo(key) => key.public_key().to_commitment(),
            NativeAuthSecretKey::EcdsaK256Keccak(key) => key.public_key().to_commitment(),
            _ => todo!("auth scheme currently not supported"),
        }
    }

    #[bindings(getter)]
    pub fn public_key(&self) -> PublicKey {
        self.0.public_key().into()
    }

    #[bindings(getter)]
    pub fn get_public_key_as_word(&self) -> Word {
        self.public_key_commitment().into()
    }

    pub fn sign(&self, message: &Word) -> Signature {
        self.sign_data(&SigningInputs::new_blind(message))
    }

    #[bindings]
    pub fn sign_data(&self, signing_inputs: &SigningInputs) -> Signature {
        let native_word = signing_inputs.to_commitment().into();
        (self.0.sign(native_word)).into()
    }

    #[bindings(js_name = "getRpoFalcon512SecretKeyAsFelts")]
    pub fn get_rpo_falcon_512_secret_key_as_felts(&self) -> Vec<Felt> {
        let secret_key_as_bytes = match &self.0 {
            NativeAuthSecretKey::Falcon512Rpo(key) => key.to_bytes(),
            _ => todo!(), // TODO: what to do with other cases
        };

        let secret_key_as_native_felts = secret_key_as_bytes
            .iter()
            .map(|a| NativeFelt::new(u64::from(*a)))
            .collect::<Vec<NativeFelt>>();

        secret_key_as_native_felts.into_iter().map(Into::into).collect()
    }

    /// Returns the ECDSA k256 Keccak secret key bytes encoded as felts.
    #[bindings(js_name = "getEcdsaK256KeccakSecretKeyAsFelts")]
    pub fn get_ecdsa_k256_keccak_secret_key_as_felts(&self) -> Vec<Felt> {
        let secret_key_as_bytes = match &self.0 {
            NativeAuthSecretKey::EcdsaK256Keccak(key) => key.to_bytes(),
            _ => todo!(), // TODO: what to do with other cases
        };

        let secret_key_as_native_felts = secret_key_as_bytes
            .iter()
            .map(|a| NativeFelt::new(u64::from(*a)))
            .collect::<Vec<NativeFelt>>();

        secret_key_as_native_felts.into_iter().map(Into::into).collect()
    }

    fn try_rng_from_seed(seed: Option<Vec<u8>>) -> platform::JsResult<StdRng> {
        match seed {
            Some(seed_bytes) => {
                let seed_array: [u8; 32] = seed_bytes
                    .try_into()
                    .map_err(|_| platform::error_from_string("Seed must be exactly 32 bytes"))?;
                Ok(StdRng::from_seed(seed_array))
            },
            None => Ok(StdRng::from_os_rng()),
        }
    }
}

// wasm-specific methods (different signatures or annotations)
#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl AuthSecretKey {
    #[wasm_bindgen(js_name = "rpoFalconWithRNG")]
    pub fn rpo_falcon_with_rng(seed: Option<Vec<u8>>) -> platform::JsResult<AuthSecretKey> {
        let mut rng = Self::try_rng_from_seed(seed)?;
        Ok(NativeAuthSecretKey::new_falcon512_rpo_with_rng(&mut rng).into())
    }

    #[wasm_bindgen(js_name = "ecdsaWithRNG")]
    pub fn ecdsa_with_rng(seed: Option<Vec<u8>>) -> platform::JsResult<AuthSecretKey> {
        let mut rng = Self::try_rng_from_seed(seed)?;
        Ok(NativeAuthSecretKey::new_ecdsa_k256_keccak_with_rng(&mut rng).into())
    }

    pub fn serialize(&self) -> Uint8Array {
        serialize_to_uint8array(&self.0)
    }

    pub fn deserialize(bytes: &Uint8Array) -> platform::JsResult<AuthSecretKey> {
        let native_secret_key = deserialize_from_uint8array::<NativeAuthSecretKey>(bytes)?;
        Ok(AuthSecretKey(native_secret_key))
    }
}

// napi-specific methods (different signatures or annotations)
#[cfg(feature = "napi")]
#[napi_derive::napi]
impl AuthSecretKey {
    #[napi(factory, js_name = "rpoFalconWithRNG")]
    pub fn rpo_falcon_with_rng(seed: Option<Vec<u8>>) -> platform::JsResult<AuthSecretKey> {
        let mut rng = Self::try_rng_from_seed(seed)?;
        Ok(NativeAuthSecretKey::new_falcon512_rpo_with_rng(&mut rng).into())
    }

    #[napi(factory, js_name = "ecdsaWithRNG")]
    pub fn ecdsa_with_rng(seed: Option<Vec<u8>>) -> platform::JsResult<AuthSecretKey> {
        let mut rng = Self::try_rng_from_seed(seed)?;
        Ok(NativeAuthSecretKey::new_ecdsa_k256_keccak_with_rng(&mut rng).into())
    }

    #[napi]
    pub fn serialize(&self) -> Buffer {
        let bytes = self.0.to_bytes();
        Buffer::from(bytes)
    }

    #[napi(factory)]
    pub fn deserialize(bytes: Buffer) -> platform::JsResult<AuthSecretKey> {
        let native_secret_key = NativeAuthSecretKey::read_from_bytes(&bytes)
            .map_err(|e| platform::error_with_context(e, "Error deserializing AuthSecretKey"))?;
        Ok(AuthSecretKey(native_secret_key))
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
