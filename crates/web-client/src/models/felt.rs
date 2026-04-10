use js_export_macro::js_export;
use miden_client::Felt as NativeFelt;

use crate::models::miden_arrays::FeltArray;
use crate::platform::{JsU64, js_u64_to_u64};

/// Field element wrapper exposed to JavaScript.
#[derive(Clone, Copy)]
#[js_export]
pub struct Felt(NativeFelt);

#[js_export]
impl Felt {
    /// Creates a new field element.
    #[js_export(constructor)]
    pub fn new(value: JsU64) -> Felt {
        Felt(NativeFelt::new(js_u64_to_u64(value)))
    }

    /// Returns the integer representation of the field element.
    #[js_export(js_name = "asInt")]
    pub fn as_int(&self) -> JsU64 {
        self.0.as_canonical_u64() as JsU64
    }

    /// Returns the string representation of the field element.
    #[js_export(js_name = "toString")]
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

/// Converts a `FeltArray` reference to a Vec of native Felt values.
pub(crate) fn felt_array_to_native_vec(felt_array: &FeltArray) -> Vec<NativeFelt> {
    Vec::from(felt_array).into_iter().map(Into::into).collect()
}

impl_napi_from_value!(Felt);
