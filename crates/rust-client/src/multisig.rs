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

pub struct MultisigClient<AUTH: TransactionAuthenticator + Sync + 'static> {
    client: Client<AUTH>,
}

impl<AUTH: TransactionAuthenticator + Sync + 'static> MultisigClient<AUTH> {
    pub fn new(client: Client<AUTH>) -> Self {
        Self { client }
    }
}

impl<AUTH: TransactionAuthenticator + Sync + 'static> Deref for MultisigClient<AUTH> {
    type Target = Client<AUTH>;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

impl<AUTH: TransactionAuthenticator + Sync + 'static> DerefMut for MultisigClient<AUTH> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.client
    }
}

impl<AUTH: TransactionAuthenticator + Sync + 'static> MultisigClient<AUTH> {
    pub fn setup_account(&mut self, approvers: Vec<PublicKey>, threshold: u32) -> (Account, Word) {
        let mut init_seed = [0u8; 32];
        self.rng().fill_bytes(&mut init_seed);

        let multisig_auth_component = AuthMultisigRpoFalcon512::new(threshold, approvers).unwrap();
        let (multisig_account, seed) = AccountBuilder::new(init_seed)
            .with_auth_component(multisig_auth_component)
            .account_type(AccountType::RegularAccountImmutableCode)
            .storage_mode(AccountStorageMode::Public)
            .with_component(BasicWallet)
            .build()
            .unwrap();

        (multisig_account, seed)
    }
}

impl<AUTH: TransactionAuthenticator + Sync + 'static> MultisigClient<AUTH> {
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
            account.storage().get_item(0).unwrap().as_elements()[1].try_into().unwrap();

        for i in 0..num_approvers as usize {
            let pub_key_index_word = Word::from([Felt::from(i as u32), ZERO, ZERO, ZERO]);
            let pub_key = account.storage().get_map_item(1, pub_key_index_word).unwrap();
            let sig_key = Hasher::merge(&[pub_key, msg]);
            if let Some(signature) = signatures.get(i).and_then(|s| s.as_ref()) {
                advice_inputs.extend(vec![(sig_key, signature.clone())]);
            }
        }

        // TODO as sanity check we should verify that we have enough signatures

        self.new_transaction(account.id(), transaction_request).await
    }
}

#[cfg(test)]
mod tests {
    use alloc::boxed::Box;

    use miden_objects::note::NoteType;
    use miden_tx::auth::SigningInputs;

    use super::*;
    use crate::testing::common::{
        TestClientKeyStore,
        insert_new_fungible_faucet,
        insert_new_wallet,
        mint_note,
    };
    use crate::testing::mock::MockRpcApi;
    use crate::tests::create_test_client;
    use crate::transaction::TransactionRequestBuilder;

    type TestMultisigClient = MultisigClient<TestClientKeyStore>;

    async fn setup_multisig_client() -> (TestMultisigClient, MockRpcApi, TestClientKeyStore) {
        let (client, mock_rpc_api, keystore) = create_test_client().await;
        (MultisigClient::new(client), mock_rpc_api, keystore)
    }

    #[tokio::test]
    async fn multisig() {
        let (mut signer_a_client, _, authenticator_a) = create_test_client().await;
        let (mut signer_b_client, _, authenticator_b) = create_test_client().await;

        let (mut coordinator_client, mock_rpc_api, coordinator_keystore) =
            setup_multisig_client().await;

        let (_, _, secret_key_a) =
            insert_new_wallet(&mut signer_a_client, AccountStorageMode::Private, &authenticator_a)
                .await
                .unwrap();
        let pub_key_a = secret_key_a.public_key();

        let (_, _, secret_key_b) =
            insert_new_wallet(&mut signer_b_client, AccountStorageMode::Private, &authenticator_b)
                .await
                .unwrap();
        let pub_key_b = secret_key_b.public_key();

        let (multisig_account, seed) =
            coordinator_client.setup_account(vec![pub_key_a, pub_key_b], 2);

        coordinator_client
            .add_account(&multisig_account, Some(seed), false)
            .await
            .unwrap();

        // we insert the faucet to the coordinator client for convenience
        let (faucet_account, ..) = insert_new_fungible_faucet(
            coordinator_client.deref_mut(),
            AccountStorageMode::Public,
            &coordinator_keystore,
        )
        .await
        .unwrap();

        // mint a note to the multisig account
        let (_tx_id, note) = mint_note(
            &mut coordinator_client,
            multisig_account.id(),
            faucet_account.id(),
            NoteType::Public,
        )
        .await;

        mock_rpc_api.prove_block();
        // TODO why do we need a second `prove_block`?
        mock_rpc_api.prove_block();
        coordinator_client.sync_state().await.unwrap();

        coordinator_client
            .import_note(miden_objects::note::NoteFile::NoteId(note.id()))
            .await
            .unwrap();

        // create a transaction to consume the note by the multisig account
        let salt = Word::empty();
        let tx_request = TransactionRequestBuilder::new()
            .auth_arg(salt)
            .build_consume_notes(vec![note.id()])
            // .build()
            .unwrap();

        // Propose the transaction (should fail with Unauthorized)
        let tx_summary = coordinator_client
            .propose_multisig_transaction(multisig_account.id(), tx_request.clone())
            .await
            .unwrap();

        let signing_inputs = SigningInputs::TransactionSummary(Box::new(tx_summary.clone()));

        let signature_a =
            authenticator_a.get_signature(pub_key_a.into(), &signing_inputs).await.unwrap();
        let signature_b =
            authenticator_b.get_signature(pub_key_b.into(), &signing_inputs).await.unwrap();

        let tx_result = coordinator_client
            .new_multisig_transaction(
                multisig_account,
                tx_request,
                tx_summary,
                vec![Some(signature_a), Some(signature_b)],
            )
            .await;

        assert!(tx_result.is_ok());
    }
}
