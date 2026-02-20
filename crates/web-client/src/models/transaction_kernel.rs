use miden_client::assembly::CodeBuilder as NativeCodeBuilder;

use crate::prelude::*;

use crate::models::code_builder::CodeBuilder;

/// Access to the default transaction kernel assembler.
#[bindings]
pub struct TransactionKernel;

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl TransactionKernel {
    /// Returns a CodeBuilder with a fresh source manager.
    pub fn assembler() -> CodeBuilder {
        let native_code_builder = NativeCodeBuilder::new();
        CodeBuilder::from_source_manager(native_code_builder.source_manager().clone())
    }
}

#[cfg(feature = "napi")]
#[napi_derive::napi]
impl TransactionKernel {
    /// Returns a CodeBuilder with a fresh source manager.
    ///
    /// Prefer using the client's `createCodeBuilder` method instead, which shares the
    /// same source manager across compilations.
    #[napi(factory)]
    pub fn assembler() -> CodeBuilder {
        let native_code_builder = NativeCodeBuilder::new();
        CodeBuilder::from_source_manager(native_code_builder.source_manager().clone())
    }
}
