use alloc::boxed::Box;
use alloc::vec::Vec;

use miden_lib::account::auth::AuthRpoFalcon512;
use miden_lib::testing::mock_account::MockAccountExt;
use miden_objects::account::{Account, AccountFile, AuthSecretKey};
use miden_objects::crypto::dsa::rpo_falcon512::{PublicKey, SecretKey};
use miden_objects::testing::account_id::{
    ACCOUNT_ID_PRIVATE_FUNGIBLE_FAUCET,
    ACCOUNT_ID_PUBLIC_FUNGIBLE_FAUCET,
};
use miden_objects::{EMPTY_WORD, Word, ZERO};

use crate::tests::create_test_client;

fn create_account_data(account_id: u128) -> AccountFile {
    let account = Account::mock(account_id, AuthRpoFalcon512::new(PublicKey::new(EMPTY_WORD)));

    AccountFile::new(
        account.clone(),
        Some(Word::default()),
        vec![AuthSecretKey::RpoFalcon512(SecretKey::new())],
    )
}

pub fn create_initial_accounts_data() -> Vec<AccountFile> {
    let account = create_account_data(ACCOUNT_ID_PRIVATE_FUNGIBLE_FAUCET);

    let faucet_account = create_account_data(ACCOUNT_ID_PUBLIC_FUNGIBLE_FAUCET);

    // Create Genesis state and save it to a file
    let accounts = vec![account, faucet_account];

    accounts
}

#[tokio::test]
pub async fn try_add_account() {
    // generate test client
    let (mut client, _rpc_api, _) = Box::pin(create_test_client()).await;

    let account = Account::mock(
        ACCOUNT_ID_PRIVATE_FUNGIBLE_FAUCET,
        AuthRpoFalcon512::new(PublicKey::new(EMPTY_WORD)),
    );

    // The mock account has nonce 1, we need it to be 0 for the test.
    let (id, vault, storage, code, _) = account.into_parts();
    let account = Account::from_parts(id, vault, storage, code, ZERO);

    assert!(client.add_account(&account, None, false).await.is_err());
    assert!(client.add_account(&account, Some(Word::default()), false).await.is_ok());
}

#[tokio::test]
async fn load_accounts_test() {
    // generate test client
    let (mut client, ..) = Box::pin(create_test_client()).await;

    let created_accounts_data = create_initial_accounts_data();

    for account_data in created_accounts_data.clone() {
        client
            .add_account(&account_data.account, account_data.account_seed, false)
            .await
            .unwrap();
    }

    let expected_accounts: Vec<Account> = created_accounts_data
        .into_iter()
        .map(|account_data| account_data.account)
        .collect();
    let accounts = client.get_account_headers().await.unwrap();

    assert_eq!(accounts.len(), 2);
    for (client_acc, expected_acc) in accounts.iter().zip(expected_accounts.iter()) {
        assert_eq!(client_acc.0.commitment(), expected_acc.commitment());
    }
}
