use miden_client::Felt as NativeFelt;
use miden_client::crypto::Rpo256 as NativeRpo256;
use wasm_bindgen::prelude::*;

use super::felt::{Felt, FeltArray};
use super::word::Word;

#[wasm_bindgen]
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
