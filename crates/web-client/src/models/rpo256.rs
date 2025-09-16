use miden_objects::Felt as NativeFelt;
use miden_objects::crypto::hash::rpo::Rpo256 as NativeRpo256;
use wasm_bindgen::prelude::*;

use super::felt::Felt;
use super::word::Word;
use crate::models::miden_arrays::FeltArray;

#[wasm_bindgen]
#[derive(Copy, Clone)]
pub struct Rpo256;

#[wasm_bindgen]
impl Rpo256 {
    #[wasm_bindgen(js_name = "hashElements")]
    pub fn hash_elements(felt_array: &FeltArray) -> Word {
        let felts: Vec<Felt> = felt_array.into();
        let native_felts: Vec<NativeFelt> = felts.iter().map(Into::into).collect();

        let native_digest = NativeRpo256::hash_elements(&native_felts);

        native_digest.into()
    }
}
