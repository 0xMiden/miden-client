use alloc::sync::Arc;

use miden_client::account::AccountComponentCode as NativeAccountComponentCode;
use miden_client::assembly::{
    Assembler, CodeBuilder as NativeCodeBuilder, Library as NativeLibrary, Module, ModuleKind,
    Path, PrintDiagnostic, Report, SourceManagerSync,
};
use wasm_bindgen::prelude::*;

use crate::js_error_with_context;
use crate::models::account_component_code::AccountComponentCode;
use crate::models::library::Library;
use crate::models::note_script::NoteScript;
use crate::models::transaction_script::TransactionScript;

/// Utility for linking libraries and compiling transaction/note scripts.
#[derive(Clone)]
#[wasm_bindgen(inspectable)]
pub struct CodeBuilder {
    // Keep the builder and derive an assembler when compiling account component code.
    builder: NativeCodeBuilder,
}

#[wasm_bindgen]
impl CodeBuilder {
    pub(crate) fn from_source_manager(source_manager: Arc<dyn SourceManagerSync>) -> Self {
        let builder = NativeCodeBuilder::with_source_manager(source_manager);
        Self { builder }
    }

    /// Given a module path (something like `my_lib::module`) and source code, this will
    /// statically link it for use with scripts to be built with this builder.
    #[wasm_bindgen(js_name = "linkModule")]
    pub fn link_module(&mut self, module_path: &str, module_code: &str) -> Result<(), JsValue> {
        self.builder.link_module(module_path, module_code).map_err(|e| {
            js_error_with_context(
                e,
                &format!("script builder: failed to link module with path {module_path}"),
            )
        })?;
        Ok(())
    }

    /// Statically links the given library.
    ///
    /// Static linking means the library code is copied into the script code.
    /// Use this for most libraries that are not available on-chain.
    ///
    /// Receives as argument the library to link.
    #[wasm_bindgen(js_name = "linkStaticLibrary")]
    pub fn link_static_library(&mut self, library: &Library) -> Result<(), JsValue> {
        let library: NativeLibrary = library.into();
        self.builder.link_static_library(&library).map_err(|e| {
            js_error_with_context(e, "script builder: failed to link static library")
        })?;
        Ok(())
    }

    /// This is useful to dynamically link the {@link Library} of a foreign account
    /// that is invoked using foreign procedure invocation (FPI). Its code is available
    /// on-chain and so it does not have to be copied into the script code.
    ///
    /// For all other use cases not involving FPI, link the library statically.
    /// Receives as argument the library to be linked.
    #[wasm_bindgen(js_name = "linkDynamicLibrary")]
    pub fn link_dynamic_library(&mut self, library: &Library) -> Result<(), JsValue> {
        let library: NativeLibrary = library.into();
        self.builder.link_dynamic_library(&library).map_err(|e| {
            js_error_with_context(e, "script builder: failed to link dynamic library")
        })?;
        Ok(())
    }

    /// Given a Transaction Script's source code, compiles it with the available
    /// modules under this builder. Returns the compiled script.
    #[wasm_bindgen(js_name = "compileTxScript")]
    pub fn compile_tx_script(&self, tx_script: &str) -> Result<TransactionScript, JsValue> {
        // Sadly, the compile function below would take ownership of self.
        // If this function were to take self by ownership instead of reference,
        // it would leave the JS side with a null value on, so we have to clone it to compile
        // the given program.
        let cloned = self.builder.clone();
        let compiled_tx_script = cloned
            .compile_tx_script(tx_script)
            .map_err(|err| js_error_with_context(err, "failed to compile transaction script"))?;
        Ok(compiled_tx_script.into())
    }

    /// Given a Note Script's source code, compiles it with the available
    /// modules under this builder. Returns the compiled script.
    #[wasm_bindgen(js_name = "compileNoteScript")]
    pub fn compile_note_script(&self, program: &str) -> Result<NoteScript, JsValue> {
        // This clone is explained under compile_tx_script
        let builder = self.builder.clone();
        let tx_script = builder
            .compile_note_script(program)
            .map_err(|err| js_error_with_context(err, "failed to compile note script"))?;
        Ok(tx_script.into())
    }

    /// Given a Library Path, and a source code, turn it into a Library.
    /// E.g. A path library can be `miden::my_contract`. When turned into a library,
    /// this can be used from another script with an import statement, following the
    /// previous example: `use miden::my_contract'.
    #[wasm_bindgen(js_name = "buildLibrary")]
    pub fn build_library(&self, library_path: &str, source_code: &str) -> Result<Library, JsValue> {
        let library_path = Path::validate(library_path).map_err(|e| {
            js_error_with_context(
                e,
                &format!("script builder: failed to build library -- invalid path {library_path}"),
            )
        })?;
        let module = Module::parser(ModuleKind::Library)
            .parse_str(library_path, source_code, self.builder.source_manager())
            .map_err(|e| {
                let err_msg = format_assembler_error(&e, "error while parsing module");
                JsValue::from(err_msg)
            })?;

        let assembler: Assembler = self.builder.clone().into();
        let native_library_build = assembler.assemble_library([module]);
        match native_library_build {
            Ok(native_library) => Ok(native_library.into()),
            Err(error_report) => {
                let err_msg =
                    format_assembler_error(&error_report, "error while assembling library");
                Err(JsValue::from(err_msg))
            },
        }
    }

    /// Given an `AccountComponentCode`, compiles it
    /// with the available modules under this builder. Returns the compiled account component code.
    #[wasm_bindgen(js_name = "compileAccountComponentCode")]
    pub fn compile_account_component_code(
        &self,
        account_code: &str,
    ) -> Result<AccountComponentCode, JsValue> {
        let assembler: Assembler = self.builder.clone().into();
        let native_library = assembler.assemble_library([account_code]).map_err(|e| {
            JsValue::from_str(&format!(
                "Failed to compile account component:
        {e}"
            ))
        })?;

        let native_code: NativeAccountComponentCode = native_library.into();
        Ok(native_code.into())
    }
}

// HELPERS
// ================================================================================================
// The assembler type returns a miette::Report instead of an Err, so this
// takes the report and returns it as an error.
fn format_assembler_error(err_report: &Report, extra_context: &str) -> String {
    let error = PrintDiagnostic::new(&err_report);

    format!("script builder error {extra_context}: {error} ")
}
