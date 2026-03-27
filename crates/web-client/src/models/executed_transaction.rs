use js_export_macro::js_export;
use miden_client::account::AccountHeader as NativeAccountHeader;
use miden_client::transaction::ExecutedTransaction as NativeExecutedTransaction;

use super::account_delta::AccountDelta;
use super::account_header::AccountHeader;
use super::account_id::AccountId;
use super::block_header::BlockHeader;
use super::input_notes::InputNotes;
use super::output_notes::OutputNotes;
use super::transaction_args::TransactionArgs;
use super::transaction_id::TransactionId;

/// Describes the result of executing a transaction program for the Miden protocol.
///
/// Executed transaction serves two primary purposes:
/// - It contains a complete description of the effects of the transaction. Specifically, it
///   contains all output notes created as the result of the transaction and describes all the
///   changes made to the involved account (i.e., the account delta).
/// - It contains all the information required to re-execute and prove the transaction in a
///   stateless manner. This includes all public transaction inputs, but also all nondeterministic
///   inputs that the host provided to Miden VM while executing the transaction (i.e., advice
///   witness).
#[derive(Clone)]
#[js_export]
pub struct ExecutedTransaction(NativeExecutedTransaction);

#[js_export]
impl ExecutedTransaction {
    /// Returns the transaction ID.
    pub fn id(&self) -> TransactionId {
        self.0.id().into()
    }

    /// Returns the account the transaction was executed against.
    #[js_export(js_name = "accountId")]
    pub fn account_id(&self) -> AccountId {
        self.0.account_id().into()
    }

    //TODO: Expose partial account
    /// Returns the initial account header before execution.
    #[js_export(js_name = "initialAccountHeader")]
    pub fn initial_account_header(&self) -> AccountHeader {
        NativeAccountHeader::from(self.0.initial_account()).into()
    }

    /// Returns the final account header after execution.
    #[js_export(js_name = "finalAccountHeader")]
    pub fn final_account_header(&self) -> AccountHeader {
        self.0.final_account().into()
    }

    /// Returns the input notes consumed by the transaction.
    #[js_export(js_name = "inputNotes")]
    pub fn input_notes(&self) -> InputNotes {
        self.0.input_notes().into()
    }

    /// Returns the output notes produced by the transaction.
    #[js_export(js_name = "outputNotes")]
    pub fn output_notes(&self) -> OutputNotes {
        self.0.output_notes().into()
    }

    /// Returns the arguments passed to the transaction script.
    #[js_export(js_name = "txArgs")]
    pub fn tx_args(&self) -> TransactionArgs {
        self.0.tx_args().into()
    }

    /// Returns the block header that included the transaction.
    #[js_export(js_name = "blockHeader")]
    pub fn block_header(&self) -> BlockHeader {
        self.0.block_header().into()
    }

    /// Returns the account delta resulting from execution.
    #[js_export(js_name = "accountDelta")]
    pub fn account_delta(&self) -> AccountDelta {
        self.0.account_delta().into()
    }

    // TODO: tx_inputs

    // TODO: advice_witness
}

// CONVERSIONS
// ================================================================================================

impl From<NativeExecutedTransaction> for ExecutedTransaction {
    fn from(native_executed_transaction: NativeExecutedTransaction) -> Self {
        ExecutedTransaction(native_executed_transaction)
    }
}

impl From<&NativeExecutedTransaction> for ExecutedTransaction {
    fn from(native_executed_transaction: &NativeExecutedTransaction) -> Self {
        ExecutedTransaction(native_executed_transaction.clone())
    }
}
