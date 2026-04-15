use miden_client::Felt as NativeFelt;
use miden_client::crypto::Poseidon2 as NativePoseidon2;
use wasm_bindgen::prelude::*;

use super::felt::Felt;
use super::word::Word;
use crate::models::miden_arrays::FeltArray;

/// Poseidon2 hashing helpers exposed to JavaScript.
#[wasm_bindgen]
#[derive(Copy, Clone)]
pub struct Poseidon2;

#[wasm_bindgen]
impl Poseidon2 {
    /// Computes a Poseidon2 digest from the provided field elements.
    #[wasm_bindgen(js_name = "hashElements")]
    pub fn hash_elements(felt_array: &FeltArray) -> Word {
        let felts: Vec<Felt> = felt_array.into();
        let native_felts: Vec<NativeFelt> = felts.iter().map(Into::into).collect();

        let native_digest = NativePoseidon2::hash_elements(&native_felts);

        native_digest.into()
    }
}
