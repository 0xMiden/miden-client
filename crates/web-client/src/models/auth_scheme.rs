use miden_client::auth::AuthSchemeId as NativeAuthSchemeId;

use crate::prelude::*;

/// Authentication scheme identifier.
#[bindings(wasm(derive(Clone, Copy)), napi(string_enum))]
#[derive(Debug, Eq, PartialEq)]
pub enum AuthScheme {
    AuthRpoFalcon512,
    AuthEcdsaK256Keccak,
}

// Compile-time check to ensure both enums stay aligned (only for wasm where repr(u8) is used).
#[cfg(feature = "wasm")]
const _: () = {
    assert!(NativeAuthSchemeId::Falcon512Rpo as u8 == AuthScheme::AuthRpoFalcon512 as u8);
    assert!(NativeAuthSchemeId::EcdsaK256Keccak as u8 == AuthScheme::AuthEcdsaK256Keccak as u8);
};

impl TryFrom<AuthScheme> for NativeAuthSchemeId {
    type Error = platform::PlatformError;

    fn try_from(value: AuthScheme) -> Result<Self, Self::Error> {
        match value {
            AuthScheme::AuthRpoFalcon512 => Ok(NativeAuthSchemeId::Falcon512Rpo),
            AuthScheme::AuthEcdsaK256Keccak => Ok(NativeAuthSchemeId::EcdsaK256Keccak),
        }
    }
}

impl TryFrom<NativeAuthSchemeId> for AuthScheme {
    type Error = platform::PlatformError;

    fn try_from(value: NativeAuthSchemeId) -> Result<Self, Self::Error> {
        match value {
            NativeAuthSchemeId::Falcon512Rpo => Ok(AuthScheme::AuthRpoFalcon512),
            NativeAuthSchemeId::EcdsaK256Keccak => Ok(AuthScheme::AuthEcdsaK256Keccak),
            _ => Err(platform::error_from_string(&format!(
                "unsupported auth scheme: {value:?}"
            ))),
        }
    }
}
