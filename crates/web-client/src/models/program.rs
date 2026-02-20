use miden_client::vm::Program as NativeProgram;

use crate::prelude::*;

/// A compiled VM program.
#[derive(Clone)]
#[bindings]
pub struct Program(NativeProgram);

// CONVERSIONS
// ================================================================================================

impl From<NativeProgram> for Program {
    fn from(native_program: NativeProgram) -> Self {
        Program(native_program)
    }
}

impl From<&NativeProgram> for Program {
    fn from(native_program: &NativeProgram) -> Self {
        Program(native_program.clone())
    }
}

impl From<Program> for NativeProgram {
    fn from(program: Program) -> Self {
        program.0
    }
}

impl From<&Program> for NativeProgram {
    fn from(program: &Program) -> Self {
        program.0.clone()
    }
}
