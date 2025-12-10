use miden_client::vm::AdviceInputs as NativeAdviceInputs;
use wasm_bindgen::prelude::*;

use super::felt::Felt;
use super::word::Word;

/// Advice inputs provided to a transaction or note script.
#[derive(Clone)]
#[wasm_bindgen]
pub struct AdviceInputs(NativeAdviceInputs);

#[wasm_bindgen]
impl AdviceInputs {
    // TODO: Constructors

    // TODO: Public Mutators

    // TODO: Destructors

    /// Returns the stack inputs as a vector of felts.
    pub fn stack(&self) -> Vec<Felt> {
        self.0.stack.iter().map(Into::into).collect()
    }

    /// Returns mapped values for a given key if present.
    #[wasm_bindgen(js_name = "mappedValues")]
    pub fn mapped_values(&self, key: &Word) -> Option<Vec<Felt>> {
        let native_key: miden_client::Word = key.into();
        self.0
            .map
            .get(&native_key)
            .map(|arc| arc.iter().copied().map(Into::into).collect())
    }

    // TODO: Merkle Store
}

// CONVERSIONS
// ================================================================================================

impl From<NativeAdviceInputs> for AdviceInputs {
    fn from(native_advice_inputs: NativeAdviceInputs) -> Self {
        AdviceInputs(native_advice_inputs)
    }
}

impl From<&NativeAdviceInputs> for AdviceInputs {
    fn from(native_advice_inputs: &NativeAdviceInputs) -> Self {
        AdviceInputs(native_advice_inputs.clone())
    }
}
