use miden_client::auth::AuthSecretKey as NativeAuthSecretKey;
use miden_client::utils::Serializable;
use miden_client::{Felt as NativeFelt, Word as NativeWord};
use wasm_bindgen::prelude::*;

use super::felt::Felt;
use super::word::Word;

#[derive(Clone, Debug)]
#[wasm_bindgen]
pub struct AuthSecretKey(NativeAuthSecretKey);

#[wasm_bindgen]
impl AuthSecretKey {
    /// Returns the public key commitment associated with this secret key.
    fn public_key_commitment(&self) -> NativeWord {
        match &self.0 {
            NativeAuthSecretKey::RpoFalcon512(key) => key.public_key().to_commitment(),
            NativeAuthSecretKey::EcdsaK256Keccak(key) => key.public_key().to_commitment(),
            _ => todo!("auth scheme currently not supported"),
        }
    }

    /// Returns the public key commitment as a word.
    #[wasm_bindgen(js_name = "getPublicKeyAsWord")]
    pub fn get_public_key_as_word(&self) -> Word {
        self.public_key_commitment().into()
    }

    /// Returns the `RpoFalcon512` secret key bytes encoded as felts.
    #[wasm_bindgen(js_name = "getRpoFalcon512SecretKeyAsFelts")]
    pub fn get_rpo_falcon_512_secret_key_as_felts(&self) -> Vec<Felt> {
        let secret_key_as_bytes = match &self.0 {
            NativeAuthSecretKey::RpoFalcon512(key) => key.to_bytes(),
            _ => todo!(), // TODO: what to do with other cases
        };

        let secret_key_as_native_felts = secret_key_as_bytes
            .iter()
            .map(|a| NativeFelt::new(u64::from(*a)))
            .collect::<Vec<NativeFelt>>();

        secret_key_as_native_felts.into_iter().map(Into::into).collect()
    }

    /// Returns the ECDSA k256 Keccak secret key bytes encoded as felts.
    #[wasm_bindgen(js_name = "getEcdsaK256KeccakSecretKeyAsFelts")]
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
