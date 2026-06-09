//! Program executor used by the client's DAP debugging path.
//!
//! The transaction executor is generic over the VM program executor. This wrapper selects the
//! debug-aware executor used by
//! [`Client::execute_program_with_dap`](crate::Client::execute_program_with_dap), allowing a DAP
//! client to attach before execution, set breakpoints, step through the transaction script, inspect
//! VM state, and request restart without changing the normal transaction setup.

use miden_processor::advice::AdviceInputs;
use miden_processor::{
    ExecutionError,
    ExecutionOptions,
    ExecutionOutput,
    FutureMaybeSend,
    Host,
    Program,
    StackInputs,
};
use miden_tx::ProgramExecutor;

/// [`ProgramExecutor`] adapter for [`miden_debug::DapExecutor`].
pub struct DapProgramExecutor(miden_debug::DapExecutor);

impl ProgramExecutor for DapProgramExecutor {
    fn new(
        stack_inputs: StackInputs,
        advice_inputs: AdviceInputs,
        options: ExecutionOptions,
    ) -> Self {
        Self(miden_debug::DapExecutor::new(stack_inputs, advice_inputs, options))
    }

    fn execute<H: Host + Send>(
        self,
        program: &Program,
        host: &mut H,
    ) -> impl FutureMaybeSend<Result<ExecutionOutput, ExecutionError>> {
        self.0.execute_async(program, host)
    }
}
