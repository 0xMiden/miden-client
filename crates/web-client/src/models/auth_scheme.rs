use miden_objects::account::auth::AuthScheme as NativeAuthScheme;
use wasm_bindgen::prelude::*;

/// Authentication schemes supported by the web client.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[wasm_bindgen]
pub enum AuthScheme {
    #[wasm_bindgen(js_name = "AuthRpoFalcon512")]
    AuthRpoFalcon512 = NativeAuthScheme::RpoFalcon512 as u8,
    #[wasm_bindgen(js_name = "AuthEcdsaK256Keccak")]
    AuthEcdsaK256Keccak = NativeAuthScheme::EcdsaK256Keccak as u8,
}

impl From<AuthScheme> for NativeAuthScheme {
    fn from(value: AuthScheme) -> Self {
        match value {
            AuthScheme::AuthRpoFalcon512 => NativeAuthScheme::RpoFalcon512,
            AuthScheme::AuthEcdsaK256Keccak => NativeAuthScheme::EcdsaK256Keccak,
        }
    }
}

impl From<NativeAuthScheme> for AuthScheme {
    fn from(value: NativeAuthScheme) -> Self {
        match value {
            NativeAuthScheme::RpoFalcon512 => AuthScheme::AuthRpoFalcon512,
            NativeAuthScheme::EcdsaK256Keccak => AuthScheme::AuthEcdsaK256Keccak,
        }
    }
}
