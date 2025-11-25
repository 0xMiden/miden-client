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

const _: () = {
    assert!(NativeAuthScheme::RpoFalcon512 as u8 == AuthScheme::AuthRpoFalcon512 as u8);
    assert!(NativeAuthScheme::EcdsaK256Keccak as u8 == AuthScheme::AuthEcdsaK256Keccak as u8);
};

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
            _ => unreachable!("unsupported auth scheme: {value:?}"),
        }
    }
}
