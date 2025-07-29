use alloc::string::ToString;
use alloc::vec::Vec;
use core::ops::{Deref, DerefMut};

use miden_lib::account::auth::AuthMultisigRpoFalcon512;
use miden_lib::account::wallets::BasicWallet;
use miden_objects::account::{Account, AccountBuilder, AccountId, AccountStorageMode, AccountType};
use miden_objects::crypto::dsa::rpo_falcon512::PublicKey;
use miden_objects::transaction::TransactionSummary;
use miden_objects::{Felt, Hasher, Word, ZERO};
use miden_tx::TransactionExecutorError;
use miden_tx::auth::TransactionAuthenticator;
use rand::RngCore;

use crate::transaction::{TransactionRequest, TransactionResult};
use crate::{Client, ClientError};

pub struct MultisigClient<AUTH: TransactionAuthenticator + 'static> {
    client: Client<AUTH>,
}

impl<AUTH: TransactionAuthenticator + 'static> MultisigClient<AUTH> {
    pub fn new(client: Client<AUTH>) -> Self {
        Self { client }
    }
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
    pub fn setup_account(&mut self, approvers: Vec<PublicKey>, threshold: u32) -> Account {
        let mut init_seed = [0u8; 32];
        self.rng().fill_bytes(&mut init_seed);

        let multisig_auth_component = AuthMultisigRpoFalcon512::new(threshold, approvers).unwrap();
        let (multisig_account, _) = AccountBuilder::new(init_seed)
            .with_auth_component(multisig_auth_component)
            .account_type(AccountType::RegularAccountImmutableCode)
            .storage_mode(AccountStorageMode::Public)
            .with_component(BasicWallet)
            .build()
            .unwrap();

        multisig_account
    }
}

impl<AUTH: TransactionAuthenticator + 'static> MultisigClient<AUTH> {
    /// Propose a multisig transaction. This is expected to "dry-run" and only return
    /// `TransactionSummary`.
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

    /// Creates and executes a transaction specified by the request against the specified multisig
    /// account. It is expected to have at least `threshold` signatures from the approvers.
    pub async fn new_multisig_transaction(
        &mut self,
        account: Account,
        mut transaction_request: TransactionRequest,
        transaction_summary: TransactionSummary,
        signatures: Vec<Option<Vec<Felt>>>,
    ) -> Result<TransactionResult, ClientError> {
        // Add signatures to the advice provider
        let advice_inputs = transaction_request.advice_map_mut();
        let msg = transaction_summary.to_commitment();
        let num_approvers: u32 =
            account.storage().get_item(0).unwrap().as_elements()[0].try_into().unwrap();

        for i in 0..num_approvers as usize {
            let pub_key_index_word = Word::from([Felt::from(i as u32), ZERO, ZERO, ZERO]);
            let pub_key = account.storage().get_map_item(2, pub_key_index_word).unwrap();
            let sig_key = Hasher::merge(&[msg, pub_key]);
            if let Some(signature) = signatures.get(i).and_then(|s| s.as_ref()) {
                advice_inputs.extend(vec![(sig_key, signature.clone())]);
            }
        }

        // TODO as sanity check we should verify that we have enough signatures

        self.new_transaction(account.id(), transaction_request).await
    }
}
