use miden_protocol::vm::Program as NativeProgram;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Clone)]
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
