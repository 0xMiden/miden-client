use miden_client::assembly::{Assembler as NativeAssembler, Library as NativeLibrary};
use miden_client::note::NoteScript as NativeNoteScript;
use miden_client::transaction::TransactionScript as NativeTransactionScript;
use wasm_bindgen::prelude::*;

use crate::models::library::Library;
use crate::models::note_script::NoteScript;
use crate::models::transaction_script::TransactionScript;

#[wasm_bindgen]
#[derive(Clone)]
pub struct Assembler(NativeAssembler);

#[wasm_bindgen]
impl Assembler {
    #[wasm_bindgen(js_name = "withLibrary")]
    pub fn with_library(self, library: &Library) -> Result<Assembler, JsValue> {
        let native_lib: NativeLibrary = library.into();

        let new_native_asm = self
            .0
            .with_dynamic_library(native_lib)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        Ok(Assembler(new_native_asm))
    }

    #[wasm_bindgen(js_name = "withDebugMode")]
    pub fn with_debug_mode(mut self, yes: bool) -> Assembler {
        self.0 = self.0.with_debug_mode(yes);
        self
    }

    #[wasm_bindgen(js_name = "compileNoteScript")]
    pub fn compile_note_script(self, note_script: &str) -> Result<NoteScript, JsValue> {
        let code = self
            .0
            .clone()
            .assemble_program(note_script)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        Ok(NativeNoteScript::new(code).into())
    }

    #[wasm_bindgen(js_name = "compileTransactionScript")]
    pub fn compile_transaction_script(
        self,
        note_script: &str,
    ) -> Result<TransactionScript, JsValue> {
        let code = self
            .0
            .clone()
            .assemble_program(note_script)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        Ok(NativeTransactionScript::new(code).into())
    }
}

// CONVERSIONS
// ================================================================================================

impl From<NativeAssembler> for Assembler {
    fn from(native_assembler: NativeAssembler) -> Self {
        Assembler(native_assembler)
    }
}

impl From<&NativeAssembler> for Assembler {
    fn from(native_assembler: &NativeAssembler) -> Self {
        Assembler(native_assembler.clone())
    }
}

impl From<Assembler> for NativeAssembler {
    fn from(assembler: Assembler) -> Self {
        assembler.0
    }
}

impl From<&Assembler> for NativeAssembler {
    fn from(assembler: &Assembler) -> Self {
        assembler.0.clone()
    }
}
