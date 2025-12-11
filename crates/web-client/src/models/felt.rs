use miden_client::Felt as NativeFelt;
use wasm_bindgen::prelude::*;

use crate::models::miden_arrays::FeltArray;

/// Field element wrapper exposed to JavaScript.
#[derive(Clone, Copy)]
#[wasm_bindgen]
pub struct Felt(NativeFelt);

#[wasm_bindgen]
impl Felt {
    /// Creates a new field element from a u64 value.
    #[wasm_bindgen(constructor)]
    pub fn new(value: u64) -> Felt {
        Felt(NativeFelt::new(value))
    }

    /// Returns the integer representation of the field element.
    #[wasm_bindgen(js_name = "asInt")]
    pub fn as_int(&self) -> u64 {
        self.0.as_int()
    }

    /// Returns the string representation of the field element.
    #[wasm_bindgen(js_name = "toString")]
    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeFelt> for Felt {
    fn from(native_felt: NativeFelt) -> Self {
        Felt(native_felt)
    }
}

impl From<&NativeFelt> for Felt {
    fn from(native_felt: &NativeFelt) -> Self {
        Felt(*native_felt)
    }
}

impl From<Felt> for NativeFelt {
    fn from(felt: Felt) -> Self {
        felt.0
    }
}

impl From<&Felt> for NativeFelt {
    fn from(felt: &Felt) -> Self {
        felt.0
    }
}

// CONVERSIONS
// ================================================================================================

impl From<&FeltArray> for Vec<NativeFelt> {
    fn from(felt_array: &FeltArray) -> Self {
        felt_array.__inner.iter().map(Into::into).collect()
    }
}
