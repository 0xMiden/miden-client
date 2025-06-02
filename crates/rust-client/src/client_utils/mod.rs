use std::{boxed::Box, string::String, sync::Arc};

use miden_assembly::{Assembler, DefaultSourceManager, LibraryPath};
use miden_lib::transaction::TransactionKernel;
use miden_objects::{
    assembly::{Library, Module, ModuleKind},
    transaction::TransactionScript,
};

/// Creates a Miden library from the provided account code and library path.
///
/// # Arguments
///
/// * `account_code` - The account code written in MASM.
/// * `library_path` - The path from which to call the library procedures.
///
/// # Returns
///
/// Returns the resulting `Library` if successful, or an error if the library cannot be created.
pub fn compile_library(
    account_code: String,
    library_path: &str,
) -> Result<miden_assembly::Library, Box<dyn std::error::Error>> {
    let assembler: Assembler = TransactionKernel::assembler().with_debug_mode(true);
    let source_manager = Arc::new(DefaultSourceManager::default());

    let module = Module::parser(ModuleKind::Library).parse_str(
        LibraryPath::new(library_path)?,
        account_code,
        &source_manager,
    )?;

    let library = assembler.assemble_library([module])?;
    Ok(library)
}

/// Creates a transaction script based on the provided code and library.
///
/// # Arguments
///
/// * `script_code` - The code for the transaction script written in MASM.
/// * `library` - The library to use with the script.
///
/// # Returns
///
/// Returns a `TransactionScript` if successfully created, or an error.
pub fn compile_tx_script(
    script_code: String,
    library: Library,
) -> Result<TransactionScript, Box<dyn std::error::Error>> {
    let assembler = TransactionKernel::assembler().with_library(library)?;

    Ok(TransactionScript::compile(script_code, [], assembler)?)
}
