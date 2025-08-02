use crate::{
    Client, ClientError,
    transaction::{TransactionRequest, TransactionResult},
};
use alloc::{string::ToString, vec::Vec};
use core::ops::{Deref, DerefMut};
use miden_objects::{Felt, account::AccountId, transaction::TransactionSummary, Word};
use miden_tx::{TransactionExecutorError, auth::TransactionAuthenticator};

pub struct MultisigClient<AUTH: TransactionAuthenticator + 'static> {
    client: Client<AUTH>,
    // coordinator_api_client
}

impl<AUTH: TransactionAuthenticator + 'static> Deref for MultisigClient<AUTH> {
    type Target = Client<AUTH>;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

impl<AUTH: TransactionAuthenticator + 'static> DerefMut for MultisigClient<AUTH> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.client
    }
}

impl<AUTH: TransactionAuthenticator + 'static> MultisigClient<AUTH> {
    /// Propose a multisig transaction. This is expected to "dry-run" and only return 
    /// `TransactionSummary`.
    pub async fn propose_multisig_transaction(
        &mut self,
        account_id: AccountId,
        transaction_request: TransactionRequest,
        salt: Word,
    ) -> Result<TransactionSummary, ClientError> {
        // Before dry-running, we see if the transaction request already has auth argument passed
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

    /// Creates and executes a transaction specified by the request against the specified multisig
    /// account. It is expected to have at least `threshold` signatures from the approvers.
    pub async fn new_multisig_transaction(
        &mut self,
        account_id: AccountId,
        transaction_request: &mut TransactionRequest,
        signatures: Vec<Vec<Felt>>,
    ) -> Result<TransactionResult, ClientError> {
        // TODO need to add signatures to the advice provider
        // let mut advice_inputs = transaction_request.advice_map().as_mut() // we need something like this

        // TODO as sanity check we should verify that we have enough signatures

        self.new_transaction(account_id, transaction_request).await
    }
}
