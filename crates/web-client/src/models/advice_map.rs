use alloc::sync::Arc;

use miden_client::vm::AdviceMap as NativeAdviceMap;
use miden_client::{Felt as NativeFelt, Word as NativeWord};

use crate::prelude::*;

use super::felt::Felt;
use super::word::Word;

#[cfg(feature = "wasm")]
use crate::models::miden_arrays::FeltArray;

/// Map of advice values keyed by words for script execution.
#[bindings]
#[derive(Clone)]
pub struct AdviceMap(NativeAdviceMap);

// Shared methods (identical signatures)
#[bindings]
impl AdviceMap {
    /// Creates an empty advice map.
    #[bindings(constructor)]
    pub fn new() -> AdviceMap {
        AdviceMap(NativeAdviceMap::default())
    }

    /// Inserts a value for the given key, returning any previous value.
    #[cfg_attr(
        feature = "wasm",
        doc = "Note: In WASM, `value` should be a `FeltArray`. In NAPI, `value` should be `Vec<&Felt>`."
    )]
    #[cfg(feature = "wasm")]
    pub fn insert(&mut self, key: &Word, value: &FeltArray) -> Option<Vec<Felt>> {
        let native_key: NativeWord = key.into();
        let wrapper_felts: Vec<Felt> = value.into();
        let native_felts: Vec<NativeFelt> = wrapper_felts.into_iter().map(Into::into).collect();
        let arc_felts: Arc<[NativeFelt]> = native_felts.into();
        self.0
            .insert(native_key, arc_felts)
            .map(|arc| arc.iter().copied().map(Into::into).collect())
    }

    /// Inserts a value for the given key, returning any previous value.
    #[cfg(feature = "napi")]
    pub fn insert(&mut self, key: &Word, value: Vec<&Felt>) -> Option<Vec<Felt>> {
        let native_key: NativeWord = key.into();
        let native_felts: Vec<NativeFelt> = value.into_iter().map(|f| f.into()).collect();
        let arc_felts: Arc<[NativeFelt]> = native_felts.into();
        self.0
            .insert(native_key, arc_felts)
            .map(|arc| arc.iter().copied().map(Into::into).collect())
    }
}

// wasm-specific methods
#[cfg(feature = "wasm")]
impl AdviceMap {}

// napi-specific methods
#[cfg(feature = "napi")]
#[napi_derive::napi]
impl AdviceMap {}

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
