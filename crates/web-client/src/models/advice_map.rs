use alloc::sync::Arc;

use js_export_macro::js_export;
use miden_client::vm::AdviceMap as NativeAdviceMap;
use miden_client::{Felt as NativeFelt, Word as NativeWord};

use super::felt::Felt;
use crate::models::miden_arrays::FeltArray;
use crate::models::word::Word;

/// Map of advice values keyed by words for script execution.
#[derive(Clone)]
#[js_export]
pub struct AdviceMap(NativeAdviceMap);

#[js_export]
impl AdviceMap {
    /// Creates an empty advice map.
    #[js_export(constructor)]
    pub fn new() -> AdviceMap {
        AdviceMap(NativeAdviceMap::default())
    }

    /// Inserts a value for the given key, returning any previous value.
    pub fn insert(&mut self, key: &Word, value: &FeltArray) -> Option<Vec<Felt>> {
        let native_key: NativeWord = key.into();
        let native_felts: Vec<NativeFelt> = super::felt::felt_array_to_native_vec(value);
        let arc_felts: Arc<[NativeFelt]> = native_felts.into();
        self.0
            .insert(native_key, arc_felts)
            .map(|arc| arc.iter().copied().map(Into::into).collect())
    }
}

impl Default for AdviceMap {
    fn default() -> Self {
        Self::new()
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeAdviceMap> for AdviceMap {
    fn from(native_advice_map: NativeAdviceMap) -> Self {
        AdviceMap(native_advice_map)
    }
}

impl From<&NativeAdviceMap> for AdviceMap {
    fn from(native_advice_map: &NativeAdviceMap) -> Self {
        AdviceMap(native_advice_map.clone())
    }
}

impl From<AdviceMap> for NativeAdviceMap {
    fn from(advice_map: AdviceMap) -> Self {
        advice_map.0
    }
}

impl From<&AdviceMap> for NativeAdviceMap {
    fn from(advice_map: &AdviceMap) -> Self {
        advice_map.0.clone()
    }
}
