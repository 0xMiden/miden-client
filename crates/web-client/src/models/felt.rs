use miden_client::Felt as NativeFelt;

use crate::prelude::*;

/// Field element wrapper exposed to JavaScript.
#[bindings]
#[derive(Clone, Copy)]
pub struct Felt(#[cfg_attr(feature = "napi", allow(dead_code))] pub(crate) NativeFelt);

// All methods unified with i64 signatures
#[bindings]
impl Felt {
    /// Creates a new field element from a value.
    #[bindings(constructor)]
    pub fn new(value: i64) -> Felt {
        Felt(NativeFelt::new(value as u64))
    }

    /// Returns the integer representation of the field element.
    pub fn as_int(&self) -> i64 {
        self.0.as_int() as i64
    }

    /// Returns the string representation of the field element.
    #[bindings(js_name = "toString")]
    #[allow(clippy::inherent_to_string)]
    pub fn to_string_js(&self) -> String {
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

