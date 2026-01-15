use alloc::boxed::Box;
use alloc::vec::Vec;
use std::collections::BTreeSet;

use miden_client::auth::{
    AuthEcdsaK256Keccak,
    AuthRpoFalcon512,
    AuthSecretKey,
    PublicKeyCommitment,
};
use miden_protocol::account::{Account, AccountFile};
use miden_protocol::testing::account_id::{
    ACCOUNT_ID_PRIVATE_FUNGIBLE_FAUCET,
    ACCOUNT_ID_PUBLIC_FUNGIBLE_FAUCET,
};
use miden_protocol::{EMPTY_WORD, Word, ZERO};
use miden_standards::testing::mock_account::MockAccountExt;

use crate::tests::create_test_client;

fn create_account_data(account_id: u128) -> AccountFile {
    let account =
        Account::mock(account_id, AuthRpoFalcon512::new(PublicKeyCommitment::from(EMPTY_WORD)));

    AccountFile::new(account.clone(), vec![AuthSecretKey::new_falcon512_rpo()])
}

fn create_ecdsa_account_data(account_id: u128) -> AccountFile {
    let account =
        Account::mock(account_id, AuthEcdsaK256Keccak::new(PublicKeyCommitment::from(EMPTY_WORD)));

    AccountFile::new(account.clone(), vec![AuthSecretKey::new_falcon512_rpo()])
}

pub fn create_initial_accounts_data() -> Vec<AccountFile> {
    let account = create_account_data(ACCOUNT_ID_PRIVATE_FUNGIBLE_FAUCET);

    let faucet_account = create_account_data(ACCOUNT_ID_PUBLIC_FUNGIBLE_FAUCET);

    // Create Genesis state and save it to a file
    let accounts = vec![account, faucet_account];

    accounts
}

pub fn create_ecdsa_initial_accounts_data() -> Vec<AccountFile> {
    let account = create_ecdsa_account_data(ACCOUNT_ID_PRIVATE_FUNGIBLE_FAUCET);

    let faucet_account = create_ecdsa_account_data(ACCOUNT_ID_PUBLIC_FUNGIBLE_FAUCET);

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
        AuthRpoFalcon512::new(PublicKeyCommitment::from(EMPTY_WORD)),
    );

    // The mock account has nonce 1, we need it to be 0 for the test.
    let (id, vault, storage, code, ..) = account.into_parts();
    let account_without_seed =
        Account::new_unchecked(id, vault.clone(), storage.clone(), code.clone(), ZERO, None);
    assert!(client.add_account(&account_without_seed, false).await.is_err());

    let account_with_seed =
        Account::new_unchecked(id, vault, storage, code, ZERO, Some(Word::default()));

    assert!(client.add_account(&account_with_seed, false).await.is_ok());
}

#[tokio::test]
pub async fn try_add_ecdsa_account() {
    // generate test client
    let (mut client, _rpc_api, _) = Box::pin(create_test_client()).await;

    let account = Account::mock(
        ACCOUNT_ID_PRIVATE_FUNGIBLE_FAUCET,
        AuthEcdsaK256Keccak::new(PublicKeyCommitment::from(EMPTY_WORD)),
    );

    // The mock account has nonce 1, we need it to be 0 for the test.
    let (id, vault, storage, code, ..) = account.into_parts();
    let account_without_seed =
        Account::new_unchecked(id, vault.clone(), storage.clone(), code.clone(), ZERO, None);
    assert!(client.add_account(&account_without_seed, false).await.is_err());

    let account_with_seed =
        Account::new_unchecked(id, vault, storage, code, ZERO, Some(Word::default()));

    assert!(client.add_account(&account_with_seed, false).await.is_ok());
}

#[tokio::test]
async fn load_accounts_test() {
    // generate test client
    let (mut client, ..) = Box::pin(create_test_client()).await;

    let created_accounts_data = create_initial_accounts_data();

    for account_data in created_accounts_data.clone() {
        client.add_account(&account_data.account, false).await.unwrap();
    }

    let expected_accounts: Vec<Account> = created_accounts_data
        .into_iter()
        .map(|account_data| account_data.account)
        .collect();
    let accounts = client.get_account_headers().await.unwrap();

    assert_eq!(accounts.len(), 2);

    let actual_commitments: BTreeSet<_> =
        accounts.into_iter().map(|(header, _)| header.commitment()).collect();
    let expected_commitments: BTreeSet<_> =
        expected_accounts.into_iter().map(|account| account.commitment()).collect();

    assert_eq!(actual_commitments, expected_commitments);
}

#[tokio::test]
async fn load_ecdsa_accounts_test() {
    // generate test client
    let (mut client, ..) = Box::pin(create_test_client()).await;

    let created_accounts_data = create_ecdsa_initial_accounts_data();
    for account_data in created_accounts_data.clone() {
        client.add_account(&account_data.account, false).await.unwrap();
    }

    let expected_accounts: Vec<Account> = created_accounts_data
        .into_iter()
        .map(|account_data| account_data.account)
        .collect();
    let accounts = client.get_account_headers().await.unwrap();

    assert_eq!(accounts.len(), 2);

    let actual_commitments: BTreeSet<_> =
        accounts.into_iter().map(|(header, _)| header.commitment()).collect();
    let expected_commitments: BTreeSet<_> =
        expected_accounts.into_iter().map(|account| account.commitment()).collect();

    assert_eq!(actual_commitments, expected_commitments);
}
