use alloc::string::ToString;

use miden_client::{
    ScriptBuilder as NativeScriptBuilder,
    assembly::{Assembler, Library as NativeLibrary, LibraryPath, Module, ModuleKind, Report},
    transaction::TransactionKernel,
};
use wasm_bindgen::prelude::*;

use crate::js_error_with_context;
use crate::models::library::Library;
use crate::models::note_script::NoteScript;
use crate::models::transaction_script::TransactionScript;

#[derive(Clone)]
#[wasm_bindgen(inspectable)]
pub struct ScriptBuilder {
    // We need both a builder and an assembler  since we want the capability of linking libraries,
    // and compiling scripts. This can be done by the NativeScriptBuilder alone, but we still
    // need an assembler to compile an AccountComponent. After miden-base issue #1756 is complete
    // we will be able to remove the Assembler and only use the NativeScriptBuilder.
    builder: NativeScriptBuilder,
    assembler: Assembler,
}

#[derive(Clone, Debug)]
#[wasm_bindgen]
pub enum ScriptBuilderMode {
    Debug,
    Normal,
}

#[wasm_bindgen]
impl ScriptBuilder {
    /// Instance a `ScriptBuilder`. Will use debug mode (or not), depending
    /// on the mode passed when initially instanced.
    #[wasm_bindgen(constructor)]
    pub fn new(mode: ScriptBuilderMode) -> Self {
        let in_debug_mode = mode.into();
        let builder = NativeScriptBuilder::new(in_debug_mode);
        let assembler =
            TransactionKernel::assembler_with_source_manager(builder.source_manager().clone())
                .with_debug_mode(in_debug_mode);
        Self { builder, assembler }
    }

    #[wasm_bindgen(js_name = "linkModule")]
    pub fn link_module(&mut self, module_path: &str, module_code: &str) -> Result<(), JsValue> {
        self.builder.link_module(module_path, module_code).map_err(|e| {
            js_error_with_context(
                e,
                &format!("script builder: failed to link module with path {module_path}"),
            )
        })?;
        self.assembler.compile_and_statically_link(module_code).map_err(|e| {
            let err_msg =
                format_assembler_error(&e, "script builder: assembler failed to link module");
            JsValue::from(err_msg)
        })?;
        Ok(())
    }

    #[wasm_bindgen(js_name = "linkStaticLibrary")]
    pub fn link_static_library(&mut self, library: &Library) -> Result<(), JsValue> {
        let library: NativeLibrary = library.into();
        self.builder.link_static_library(&library).map_err(|e| {
            js_error_with_context(e, "script builder: failed to link static library")
        })?;
        self.assembler.link_static_library(&library).map_err(|e| {
            let err_msg = format_assembler_error(
                &e,
                "script builder: assembler failed to link static library",
            );
            JsValue::from_str(&err_msg)
        })?;
        Ok(())
    }

    #[wasm_bindgen(js_name = "linkDynamicLibrary")]
    pub fn link_dynamic_library(&mut self, library: &Library) -> Result<(), JsValue> {
        let library: NativeLibrary = library.into();
        self.builder.link_dynamic_library(&library).map_err(|e| {
            js_error_with_context(e, "script builder: failed to link dynamic library")
        })?;
        self.assembler.link_dynamic_library(&library).map_err(|e| {
            let err_msg = format_assembler_error(
                &e,
                "script builder: assembler failed to link dynamic library",
            );
            JsValue::from_str(&err_msg)
        })?;
        Ok(())
    }

    #[wasm_bindgen(js_name = "compileTxScript")]
    pub fn compile_tx_script(&self, tx_script: &str) -> Result<TransactionScript, JsValue> {
        // Sadly, the compile function below would take ownership of self.
        // If this function were to take self by ownership instead of reference,
        // it would leave the JS side with a null value on, so we have to clone it to compile
        // the given program.
        let cloned = self.builder.clone();
        let compiled_tx_script = cloned.compile_tx_script(tx_script).unwrap();
        Ok(compiled_tx_script.into())
    }

    #[wasm_bindgen(js_name = "compileNoteScript")]
    pub fn compile_note_script(&self, program: &str) -> Result<NoteScript, JsValue> {
        // This clone is explained under compile_tx_script
        let builder = self.builder.clone();
        let tx_script = builder
            .compile_note_script(program)
            .map_err(|err| js_error_with_context(err, "failed to compile note script"))?;
        Ok(tx_script.into())
    }

    #[wasm_bindgen(js_name = "buildLibrary")]
    pub fn build_library(&self, library_path: &str, source_code: &str) -> Result<Library, JsValue> {
        let library_path = LibraryPath::new(library_path).map_err(|e| {
            js_error_with_context(
                e, "script builder: failed to build library -- could not create library_path with path {library_path}",
            )
        })?;
        let module = Module::parser(ModuleKind::Library)
            .parse_str(library_path, source_code, self.builder.source_manager().as_ref())
            .map_err(|e| {
                let err_msg = format_assembler_error(&e, "error while parsing module");
                JsValue::from(err_msg)
            })?;

        let native_library_build = self.clone_assembler().assemble_library([module]);
        match native_library_build {
            Ok(native_library) => Ok(native_library.into()),
            Err(error_report) => {
                let err_msg =
                    format_assembler_error(&error_report, "error while assembling library");
                Err(JsValue::from(err_msg))
            },
        }
    }

    /// Returns the inner assembler . This is because multiple "compile" functions
    /// in `miden_lib` to consume an Assembler, so we need to clone the value.
    pub(crate) fn clone_assembler(&self) -> Assembler {
        self.assembler.clone()
    }
}

// HELPERS
// ================================================================================================
// The assembler type returns a miette::Report instead of an Err, so this
// takes the report and returns it as an error.
fn format_assembler_error(err_report: &Report, extra_context: &str) -> String {
    let error = err_report.chain().map(ToString::to_string).collect::<Vec<String>>().join("\n");

    format!("script builder: {error}: failed to build given library: \n {extra_context}")
}

// CONVERSIONS
// ================================================================================================
impl From<ScriptBuilderMode> for bool {
    fn from(value: ScriptBuilderMode) -> Self {
        match value {
            ScriptBuilderMode::Debug => true,
            ScriptBuilderMode::Normal => false,
        }
    }
}
