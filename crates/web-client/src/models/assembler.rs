use miden_client::transaction::TransactionScript as NativeTransactionScript;
use miden_objects::assembly::{Assembler as NativeAssembler, Library as NativeLibrary};
use miden_objects::note::NoteScript as NativeNoteScript;
use wasm_bindgen::prelude::*;

use crate::models::library::Library;
use crate::models::note_script::NoteScript;
use crate::models::transaction_script::TransactionScript;
use miden_lib::utils::ScriptBuilder as NativeScriptBuilder;
use wasm_bindgen::prelude::*;

// FIXME: Move this to its own, separate file.
#[derive(Clone)]
#[wasm_bindgen(inspectable)]
pub struct ScriptBuilder {
    // We need both a builder and an assembler
    // since we want the capability of assembling libraries,
    // which only an assembler can do. The builder does have
    // an assembler, but it's a private field currently.
    builder: NativeScriptBuilder,
    assembler: NativeAssembler,
}

#[wasm_bindgen]
impl ScriptBuilder {
    #[wasm_bindgen(constructor)]
    // FIXME: Double check if we really need both a builder and an assembler.
    pub fn new(in_debug_mode: bool) -> Self {
        let builder = NativeScriptBuilder::new(in_debug_mode);
        let assembler = NativeAssembler::new(builder.source_manager().clone());
        Self { builder, assembler }
    }

    #[wasm_bindgen(js_name = "linkModule")]
    pub fn link_module(&mut self, module_path: &str, module_code: &str) {
        // FIXME: Remove unwrap
        self.builder.link_module(module_path, module_code).unwrap();
        // FIXME: Remove unwrap
        self.assembler.compile_and_statically_link(module_code).unwrap();
    }

    // FIXME: Explain how and why would you use a static vs a dynamic library.
    #[wasm_bindgen(js_name = "linkStaticLibrary")]
    pub fn link_static_library(&mut self, library: &Library) {
        let library: NativeLibrary = library.into();
        // FIXME: Remove unwrap
        self.builder.link_static_library(&library).unwrap();
        // FIXME: Remove unwrap
        self.assembler.link_static_library(&library).unwrap();
    }

    #[wasm_bindgen(js_name = "linkDynamicLibrary")]
    pub fn link_dynamic_library(&mut self, library: &Library) {
        let library: NativeLibrary = library.into();
        // FIXME: Remove unwrap
        self.builder.link_dynamic_library(&library).unwrap();
        // FIXME: Remove unwrap
        self.assembler.link_dynamic_library(&library).unwrap();
    }

    #[wasm_bindgen(js_name = "compileTxScript")]
    pub fn compile_tx_script(&self, tx_script: &str) -> Result<TransactionScript, JsValue> {
        // FIXME: Do we want to consume the builder or not?
        let builder = self.clone();
        // FIXME: Remove unwrap
        let compiled_tx_script = builder.compile_tx_script(tx_script).unwrap();
        Ok(compiled_tx_script)
    }

    #[wasm_bindgen(js_name = "compileNoteScript")]
    pub fn compile_note_script(&self, program: &str) -> Result<TransactionScript, JsValue> {
        // FIXME: Do we want to consume the builder or not?
        let builder = self.clone();
        // FIXME: remove unwrap
        let tx_script = builder.compile_note_script(program).unwrap();
        Ok(tx_script)
    }

    #[wasm_bindgen(js_name = "buildLibrary")]
    pub fn build_library(&mut self, module: &str) -> Result<Library, JsValue> {
        // FIXME: Remove unwrap
        // FIXME: Check if we can avoid this clone (re-assign maybe?)
        let library = self.assembler.clone().assemble_library(vec![module]).unwrap();
        Ok(library.into())
    }
}

#[wasm_bindgen]
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
