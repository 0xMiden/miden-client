use crate::models::miden_arrays::FeltArray;
use miden_objects::Felt as NativeFelt;
use wasm_bindgen::prelude::*;

#[derive(Clone, Copy)]
#[wasm_bindgen]
pub struct Felt(NativeFelt);

#[wasm_bindgen]
impl Felt {
    #[wasm_bindgen(constructor)]
    pub fn new(value: u64) -> Felt {
        Felt(NativeFelt::new(value))
    }

    #[wasm_bindgen(js_name = "asInt")]
    pub fn as_int(&self) -> u64 {
        self.0.as_int()
    }

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
