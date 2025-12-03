use miden_client::Felt as NativeFelt;
use miden_client::crypto::Rpo256 as NativeRpo256;
use wasm_bindgen::prelude::*;

use super::felt::Felt;
use super::word::Word;
use crate::models::miden_arrays::FeltArray;

/// RPO256 hashing helpers exposed to JavaScript.
#[wasm_bindgen]
#[derive(Copy, Clone)]
pub struct Rpo256;

#[wasm_bindgen]
impl Rpo256 {
    /// Computes an RPO256 digest from the provided field elements.
    #[wasm_bindgen(js_name = "hashElements")]
    pub fn hash_elements(felt_array: &FeltArray) -> Word {
        let felts: Vec<Felt> = felt_array.into();
        let native_felts: Vec<NativeFelt> = felts.iter().map(Into::into).collect();

        let native_digest = NativeRpo256::hash_elements(&native_felts);

        native_digest.into()
    }
}
