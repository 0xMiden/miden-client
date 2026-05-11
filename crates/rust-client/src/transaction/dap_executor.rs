//! Adapter around `miden_debug::DapExecutor` so it slots into
//! `miden_tx::TransactionExecutor`.
//!
//! As of `miden-debug` 0.7 the crate no longer depends on `miden-tx` and so
//! its `DapExecutor` no longer implements the `miden_tx::ProgramExecutor`
//! trait directly. This newtype bridges the two: it constructs and drives
//! the upstream `DapExecutor` while presenting the trait the transaction
//! executor expects.

use miden_processor::advice::AdviceInputs;
use miden_processor::{
    ExecutionError, ExecutionOptions, ExecutionOutput, FutureMaybeSend, Host, Program, StackInputs,
};
use miden_tx::ProgramExecutor;

/// Newtype wrapper around [`miden_debug::DapExecutor`] that implements
/// [`miden_tx::ProgramExecutor`] by delegating to its `execute_async` method.
pub struct DapExecutor(miden_debug::DapExecutor);

impl ProgramExecutor for DapExecutor {
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
