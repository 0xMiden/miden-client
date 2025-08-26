use miden_client::Word;
use miden_client::account::AccountStorageMode;
use miden_client::auth::SigningInputs;
use miden_client::multisig::MultisigClient;
use miden_client::testing::common::*;
use miden_client::testing::config::ClientConfig;
use miden_client::transaction::{TransactionAuthenticator, TransactionRequestBuilder};

async fn setup_multisig_client(
    config: ClientConfig,
) -> (MultisigClient<TestClientKeyStore>, TestClientKeyStore) {
    let (client, keystore) = create_test_client(config).await;
    let multisig_client = MultisigClient::new(client);
    (multisig_client, keystore)
}

pub async fn multisig(config: ClientConfig) {
    let (mut signer_a_client, authenticator_a) = create_test_client(config.clone()).await;
    wait_for_node(&mut signer_a_client).await;
    let (mut signer_b_client, authenticator_b) = create_test_client(config.clone()).await;

    let (mut coordinator_client, _): (MultisigClient<_>, _) = setup_multisig_client(config).await;

    signer_a_client.sync_state().await.unwrap();
    signer_b_client.sync_state().await.unwrap();
    coordinator_client.sync_state().await.unwrap();

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

    let (multisig_account, seed) = coordinator_client.setup_account(vec![pub_key_a, pub_key_b], 2);

    coordinator_client
        .add_account(&multisig_account, Some(seed), false)
        .await
        .unwrap();

    let salt = Word::empty();

    let tx_request = TransactionRequestBuilder::new().auth_arg(salt).build().unwrap();
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
