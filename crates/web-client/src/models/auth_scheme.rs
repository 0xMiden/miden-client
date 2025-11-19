use wasm_bindgen::prelude::*;

/// Authentication schemes supported by the web client.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[wasm_bindgen]
pub enum AuthScheme {
    #[wasm_bindgen(js_name = "AuthRpoFalcon512")]
    AuthRpoFalcon512 = 0,
    #[wasm_bindgen(js_name = "AuthEcdsaK256Keccak")]
    AuthEcdsaK256Keccak = 1,
}
