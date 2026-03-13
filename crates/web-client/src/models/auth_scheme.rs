use core::convert::TryFrom;
use core::fmt::Debug;

use miden_client::auth::AuthSchemeId as NativeAuthSchemeId;
use js_export_macro::js_export;

use crate::platform::{JsErr, from_str_err};

/// Authentication schemes supported by the web client.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[js_export]
pub enum AuthScheme {
    AuthEcdsaK256Keccak = 1,
    AuthRpoFalcon512 = 2,
}

// Compile-time check to ensure both enums stay aligned.
const _: () = {
    assert!(NativeAuthSchemeId::Falcon512Rpo as u8 == AuthScheme::AuthRpoFalcon512 as u8);
    assert!(NativeAuthSchemeId::EcdsaK256Keccak as u8 == AuthScheme::AuthEcdsaK256Keccak as u8);
};

impl TryFrom<AuthScheme> for NativeAuthSchemeId {
    type Error = JsErr;

    fn try_from(value: AuthScheme) -> Result<Self, Self::Error> {
        match value {
            AuthScheme::AuthRpoFalcon512 => Ok(NativeAuthSchemeId::Falcon512Rpo),
            AuthScheme::AuthEcdsaK256Keccak => Ok(NativeAuthSchemeId::EcdsaK256Keccak),
        }
    }
}

impl TryFrom<NativeAuthSchemeId> for AuthScheme {
    type Error = JsErr;

    fn try_from(value: NativeAuthSchemeId) -> Result<Self, Self::Error> {
        match value {
            NativeAuthSchemeId::Falcon512Rpo => Ok(AuthScheme::AuthRpoFalcon512),
            NativeAuthSchemeId::EcdsaK256Keccak => Ok(AuthScheme::AuthEcdsaK256Keccak),
            _ => Err(unsupported_scheme_error(value)),
        }
    }
}

fn unsupported_scheme_error(scheme: impl Debug) -> JsErr {
    from_str_err(&format!("unsupported auth scheme: {scheme:?}"))
}
