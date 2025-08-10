use miden_lib::utils::ScriptBuilder as NativeScriptBuilder;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use crate::js_error_with_context;
use crate::models::{assembler::Assembler, library::Library, note_script::NoteScript, transaction_script::TransactionScript};

use miden_objects::assembly::Library as NativeLibrary;

#[wasm_bindgen]
pub struct ScriptBuilder(NativeScriptBuilder);

#[wasm_bindgen]
impl ScriptBuilder {
    #[wasm_bindgen(constructor)]
    pub fn new(in_debug_mode: bool) -> ScriptBuilder {
        ScriptBuilder(NativeScriptBuilder::new(in_debug_mode))
    }

    #[wasm_bindgen(js_name = "linkModule")]
    pub fn link_module(&mut self, module_path: &str, module_code: &str) -> Result<(), JsValue> {
        self.0
            .link_module(module_path, module_code)
            .map_err(|e| js_error_with_context(e, "failed to link module"))
    }

    #[wasm_bindgen(js_name = "linkStaticLibrary")]
    pub fn link_static_library(&mut self, library: &Library) -> Result<(), JsValue> {
        let lib: NativeLibrary = library.into();
        self.0
            .link_static_library(&lib)
            .map_err(|e| js_error_with_context(e, "failed to add static library"))
    }

    #[wasm_bindgen(js_name = "linkDynamicLibrary")]
    pub fn link_dynamic_library(&mut self, library: &Library) -> Result<(), JsValue> {
        let lib: NativeLibrary = library.into();
        self.0
            .link_dynamic_library(&lib)
            .map_err(|e| js_error_with_context(e, "failed to add dynamic library"))
    }

    #[wasm_bindgen(js_name = "compileTxScript")]
    pub fn compile_tx_script(self, script_code: &str) -> Result<TransactionScript, JsValue> {
        self.0
            .compile_tx_script(script_code)
            .map(TransactionScript::from)
            .map_err(|e| js_error_with_context(e, "failed to compile tx script"))
    }

    #[wasm_bindgen(js_name = "compileNoteScript")]
    pub fn compile_note_script(self, script_code: &str) -> Result<NoteScript, JsValue> {
        self.0
            .compile_note_script(script_code)
            .map(NoteScript::from)
            .map_err(|e| js_error_with_context(e, "failed to compile note script"))
    }
}
