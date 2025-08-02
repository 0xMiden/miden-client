use crate::{
    Client, ClientError,
    transaction::{TransactionRequest, TransactionResult},
};
use alloc::{string::ToString, vec::Vec};
use core::ops::{Deref, DerefMut};
use miden_lib::account::wallets::BasicWallet;
use miden_objects::{
    Felt, Hasher, Word, ZERO,
    account::{Account, AccountBuilder, AccountId, AccountStorageMode, AccountType},
    crypto::dsa::rpo_falcon512::PublicKey,
    transaction::TransactionSummary,
};
use miden_tx::{TransactionExecutorError, auth::TransactionAuthenticator};
use rand::RngCore;

/// Placeholder for now
pub struct MultiSigCoordinatorApiClient;

pub struct MultisigClient<AUTH: TransactionAuthenticator + 'static> {
    client: Client<AUTH>,
    coordinator_api_client: Option<MultiSigCoordinatorApiClient>, // some way of communicating with the coordinator
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
    pub fn setup_account(&self, approvers: Vec<PublicKey>, threshold: u8) -> Account {
        let mut init_seed = [0u8; 32];
        self.rng().fill_bytes(&mut init_seed);

        let multisig_auth_component = AuthMultisigRpoFalcon512::new(threshold, approvers);
        let (multisig_account, _) = AccountBuilder::new(init_seed)
            .with_auth_component(multisig_auth_component)
            .account_type(AccountType::RegularAccountImmutableCode)
            .storage_mode(AccountStorageMode::Public)
            .with_component(BasicWallet)
            .build()
            .unwrap();
        // TODO inform the coordinator that we have a new account
        // This could be a separate method, TBD
        match self.coordinator_api_client {
            Some(coordinator_api_client) => {
                // coordinator_api_client.add_account(&multisig_account).await.unwrap();
            },
            None => (), // nothing to do, no coordinator set up
        }
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
        transaction_request: TransactionRequest,
        transaction_summary: TransactionSummary,
        signatures: Vec<Option<Vec<Felt>>>,
    ) -> Result<TransactionResult, ClientError> {
        // TODO need to add signatures to the advice provider
        let mut advice_inputs = transaction_request.advice_map().as_mut(); // we need something like this
        let msg = transaction_summary.to_commitment();
        let num_approvers: u32 =
            account.storage().get_item(NUM_APPROVERS_SLOT).unwrap().as_elements()[0]
                .try_into()
                .unwrap();

        for i in 0..num_approvers {
            let pub_key_index_word = Word::from([Felt::from(i as u32), ZERO, ZERO, ZERO]);
            let pub_key = account.storage().get_map_item(KEY_MAP_SLOT, pub_key_index_word).unwrap();
            let sig_key = Hasher::merge(&[msg, pub_key]);
            if let Some(signature) = signatures[i] {
                advice_inputs.extend_advice_map(vec![(sig_key, signature)]);
            }
        }

        // TODO as sanity check we should verify that we have enough signatures

        self.new_transaction(account.id(), transaction_request).await
    }
}
