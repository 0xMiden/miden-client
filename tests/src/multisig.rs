use miden_client::{
    account::Account,
    auth::TransactionAuthenticator,
    note::{Note, build_swap_tag},
    testing::common::*,
    transaction::{SwapTransactionData, TransactionRequestBuilder},
};
use miden_objects::{
    account::AccountStorageMode,
    asset::{Asset, FungibleAsset},
    note::{NoteDetails, NoteFile, NoteType},
};

// SWAP FULLY ONCHAIN
// ================================================================================================

#[tokio::test]
async fn multisig() {
    const OFFERED_ASSET_AMOUNT: u64 = 1;
    const REQUESTED_ASSET_AMOUNT: u64 = 25;
    let (mut signer_a_client, authenticator_a) = create_test_client().await;
    wait_for_node(&mut signer_a_client).await;
    let (mut signer_b_client, authenticator_b) = create_test_client().await;

    let (mut coordinator_client, authenticator_c) = create_test_client().await;

    signer_a_client.sync_state().await.unwrap();
    signer_b_client.sync_state().await.unwrap();
    coordinator_client.sync_state().await.unwrap();

    let (account_a, _, secret_key_a) =
        insert_new_wallet(&mut signer_a_client, AccountStorageMode::Private, &authenticator_a)
            .await
            .unwrap();
    let pub_key_a = secret_key_a.public_key();

    let (account_b, _, secret_key_b) =
        insert_new_wallet(&mut signer_b_client, AccountStorageMode::Private, &authenticator_b)
            .await
            .unwrap();
    let pub_key_b = secret_key_b.public_key();

    let multisig_auth_component = AuthMultisigRpoFalcon512::new(
        2,
        vec![secret_key_a.public_key(), secret_key_b.public_key()],
    );

    let mut init_seed = [0u8; 32];
    coordinator_client.rng().fill_bytes(&mut init_seed);

    let multisig_account = AccountBuilder::new(init_seed)
        .account_type(AccountType::RegularAccountImmutableCode)
        .storage_mode(AccountStorageMode::Public)
        .with_auth_component(multisig_auth_component)
        .with_component(BasicWallet)
        .build()
        .unwrap();

    coordinator_client.add_account(&multisig_account, None, false).await.unwrap();

    let tx_request = TransactionRequestBuilder::new().build().unwrap();
    let tx_summary = coordinator_client
        .propose_multisig_transaction(multisig_account.id(), tx_request)
        .await
        .unwrap();

    let signature_a = authenticator_a.get_signature(pub_key_a, &tx_summary).unwrap();
    let signature_b = authenticator_b.get_signature(pub_key_b, &tx_summary).unwrap();

    let tx_result = coordinator_client
        .new_multisig_transaction(multisig_account.id(), tx_request, vec![signature_a, signature_b])
        .await
        .unwrap();
}
