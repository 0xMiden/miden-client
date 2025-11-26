use core::convert::TryFrom;
use core::fmt::Debug;

use miden_objects::account::auth::AuthScheme as NativeAuthScheme;
use wasm_bindgen::prelude::*;

/// Authentication schemes supported by the web client.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[wasm_bindgen]
pub enum AuthScheme {
    AuthRpoFalcon512 = 0,
    AuthEcdsaK256Keccak = 1,
}

impl TryFrom<AuthScheme> for NativeAuthScheme {
    type Error = JsValue;

    fn try_from(value: AuthScheme) -> Result<Self, Self::Error> {
        match value {
            AuthScheme::AuthRpoFalcon512 => Ok(NativeAuthScheme::RpoFalcon512),
            AuthScheme::AuthEcdsaK256Keccak => Ok(NativeAuthScheme::EcdsaK256Keccak),
        }
    }
}

impl TryFrom<NativeAuthScheme> for AuthScheme {
    type Error = JsValue;

    fn try_from(value: NativeAuthScheme) -> Result<Self, Self::Error> {
        match value {
            NativeAuthScheme::RpoFalcon512 => Ok(AuthScheme::AuthRpoFalcon512),
            NativeAuthScheme::EcdsaK256Keccak => Ok(AuthScheme::AuthEcdsaK256Keccak),
            _ => Err(unsupported_scheme_error(value)),
        }
    }
}

fn unsupported_scheme_error(scheme: impl Debug) -> JsValue {
    JsValue::from_str(&format!("unsupported auth scheme: {scheme:?}"))
}
