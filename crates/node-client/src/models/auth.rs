use miden_client::auth::{
    AuthSchemeId as NativeAuthSchemeId,
    AuthSecretKey as NativeAuthSecretKey,
};
use miden_client::utils::{Deserializable, Serializable};
use napi::bindgen_prelude::*;

use super::napi_wrap;
use super::word::Word;

// AUTH SCHEME
// ================================================================================================

#[napi(string_enum)]
#[derive(Eq, PartialEq)]
pub enum AuthScheme {
    RpoFalcon512,
    EcdsaK256Keccak,
}

impl From<AuthScheme> for NativeAuthSchemeId {
    fn from(value: AuthScheme) -> Self {
        match value {
            AuthScheme::RpoFalcon512 => NativeAuthSchemeId::Falcon512Rpo,
            AuthScheme::EcdsaK256Keccak => NativeAuthSchemeId::EcdsaK256Keccak,
        }
    }
}

impl From<&AuthScheme> for NativeAuthSchemeId {
    fn from(value: &AuthScheme) -> Self {
        match value {
            AuthScheme::RpoFalcon512 => NativeAuthSchemeId::Falcon512Rpo,
            AuthScheme::EcdsaK256Keccak => NativeAuthSchemeId::EcdsaK256Keccak,
        }
    }
}

// AUTH SECRET KEY
// ================================================================================================

napi_wrap!(clone AuthSecretKey wraps NativeAuthSecretKey);

#[napi]
impl AuthSecretKey {
    /// Generates a new RPO Falcon 512 key pair, optionally from a seed.
    #[napi]
    pub fn rpo_falcon_with_rng(seed: Option<Buffer>) -> Result<AuthSecretKey> {
        let mut rng = try_rng_from_seed(seed)?;
        Ok(NativeAuthSecretKey::new_falcon512_rpo_with_rng(&mut rng).into())
    }

    /// Generates a new ECDSA k256 Keccak key pair, optionally from a seed.
    #[napi]
    pub fn ecdsa_with_rng(seed: Option<Buffer>) -> Result<AuthSecretKey> {
        let mut rng = try_rng_from_seed(seed)?;
        Ok(NativeAuthSecretKey::new_ecdsa_k256_keccak_with_rng(&mut rng).into())
    }

    /// Returns the public key commitment as a Word.
    #[napi]
    pub fn get_public_key_as_word(&self) -> Result<Word> {
        let commitment = match &self.0 {
            NativeAuthSecretKey::Falcon512Rpo(key) => key.public_key().to_commitment(),
            NativeAuthSecretKey::EcdsaK256Keccak(key) => key.public_key().to_commitment(),
            _ => {
                return Err(napi::Error::from_reason("unsupported auth scheme"));
            },
        };
        Ok(Word(commitment))
    }

    /// Serializes the key into bytes.
    #[napi]
    pub fn serialize(&self) -> Buffer {
        self.0.to_bytes().into()
    }

    /// Deserializes a key from bytes.
    #[napi]
    pub fn deserialize(bytes: Buffer) -> Result<AuthSecretKey> {
        let native = NativeAuthSecretKey::read_from_bytes(&bytes).map_err(|err| {
            napi::Error::from_reason(format!("Failed to deserialize AuthSecretKey: {err}"))
        })?;
        Ok(AuthSecretKey(native))
    }
}

fn try_rng_from_seed(seed: Option<Buffer>) -> Result<rand::rngs::StdRng> {
    use rand::SeedableRng;
    match seed {
        Some(seed_bytes) => {
            let seed_array: [u8; 32] = seed_bytes[..]
                .try_into()
                .map_err(|_| napi::Error::from_reason("Seed must be exactly 32 bytes"))?;
            Ok(rand::rngs::StdRng::from_seed(seed_array))
        },
        None => Ok(rand::rngs::StdRng::from_os_rng()),
    }
}
