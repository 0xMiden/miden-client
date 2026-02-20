use miden_client::Felt as NativeFelt;
use miden_client::crypto::Rpo256 as NativeRpo256;

use crate::prelude::*;

use super::felt::Felt;
use super::word::Word;
#[cfg(feature = "wasm")]
use crate::models::miden_arrays::FeltArray;

/// RPO256 hashing helpers exposed to JavaScript.
#[bindings]
#[derive(Copy, Clone)]
pub struct Rpo256;

// hash_elements has different signatures: FeltArray in wasm vs Vec<&Felt> in napi
#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl Rpo256 {
    /// Computes an RPO256 digest from the provided field elements.
    
    pub fn hash_elements(felt_array: &FeltArray) -> Word {
        let felts: Vec<Felt> = felt_array.into();
        let native_felts: Vec<NativeFelt> = felts.iter().map(Into::into).collect();

        let native_digest = NativeRpo256::hash_elements(&native_felts);

        native_digest.into()
    }
}

#[cfg(feature = "napi")]
#[napi_derive::napi]
impl Rpo256 {
    /// Computes an RPO256 digest from the provided field elements.
    #[napi(factory)]
    pub fn hash_elements(felts: Vec<&Felt>) -> Word {
        let native_felts: Vec<NativeFelt> = felts.iter().map(|f| NativeFelt::from(*f)).collect();
        let native_digest = NativeRpo256::hash_elements(&native_felts);
        native_digest.into()
    }
}
