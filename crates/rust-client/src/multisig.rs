use crate::{
    Client, ClientError,
    transaction::{TransactionRequest, TransactionResult},
};
use alloc::{string::ToString, vec::Vec};
use miden_objects::{Felt, account::AccountId, transaction::TransactionSummary};
use miden_tx::{TransactionExecutorError, auth::TransactionAuthenticator};

impl<AUTH: TransactionAuthenticator + 'static> Client<AUTH> {
    /// Propose a multisig transaction.
    pub async fn propose_multisig_transaction(
        &mut self,
        account_id: AccountId,
        transaction_request: TransactionRequest,
    ) -> Result<TransactionSummary, ClientError> {
        let tx_result = self.new_transaction(account_id, transaction_request).await;

        let tx_summary = match tx_result {
            Ok(_) => {
                return Err(ClientError::MultisigTxProposalError(
                    "Expecting a dry run, but tx was executed".to_string(),
                ));
            },
            // otherwise match on Unauthorized
            Err(ClientError::TransactionExecutorError(TransactionExecutorError::Unauthorized(
                summary,
            ))) => Ok(*summary),
            Err(e) => Err(e),
        };

        tx_summary
    }

    pub async fn new_multisig_transaction(
        &mut self,
        account_id: AccountId,
        transaction_request: &mut TransactionRequest,
        signatures: Vec<Vec<Felt>>,
    ) -> Result<TransactionResult, ClientError> {
        // need to add signatures to the advice provider
        let mut advice_inputs = transaction_request.advice_map().as_mut().unwrap();

        self.new_transaction(account_id, transaction_request).await
    }
}
