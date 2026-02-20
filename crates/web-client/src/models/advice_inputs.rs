use miden_client::vm::AdviceInputs as NativeAdviceInputs;

use super::felt::Felt;
use super::word::Word;
use crate::prelude::*;

/// Advice inputs provided to a transaction or note script.
#[bindings]
#[derive(Clone)]
pub struct AdviceInputs(NativeAdviceInputs);

#[bindings]
impl AdviceInputs {
    // TODO: Constructors

    // TODO: Public Mutators

    // TODO: Destructors

    /// Returns the stack inputs as a vector of felts.
    #[bindings(getter)]
    pub fn stack(&self) -> Vec<Felt> {
        self.0.stack.iter().map(Into::into).collect()
    }

    /// Returns mapped values for a given key if present.
    #[bindings]
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
