use alloc::boxed::Box;
use alloc::vec::Vec;
use std::collections::BTreeSet;

use miden_client::auth::{AuthSchemeId, AuthSecretKey, AuthSingleSig, PublicKeyCommitment};
use miden_client::transaction::TransactionRequestBuilder;
use miden_protocol::account::{Account, AccountFile, AccountStorageMode};
use miden_protocol::asset::FungibleAsset;
use miden_protocol::note::NoteType;
use miden_protocol::testing::account_id::{
    ACCOUNT_ID_PRIVATE_FUNGIBLE_FAUCET,
    ACCOUNT_ID_PUBLIC_FUNGIBLE_FAUCET,
};
use miden_protocol::{EMPTY_WORD, Word, ZERO};
use miden_standards::testing::mock_account::MockAccountExt;

use crate::tests::{create_test_client, insert_new_fungible_faucet, insert_new_wallet};

fn create_account_data(account_id: u128) -> AccountFile {
    let account = Account::mock(
        account_id,
        AuthSingleSig::new(PublicKeyCommitment::from(EMPTY_WORD), AuthSchemeId::Falcon512Rpo),
    );

    AccountFile::new(account.clone(), vec![AuthSecretKey::new_falcon512_rpo()])
}

fn create_ecdsa_account_data(account_id: u128) -> AccountFile {
    let account = Account::mock(
        account_id,
        AuthSingleSig::new(PublicKeyCommitment::from(EMPTY_WORD), AuthSchemeId::EcdsaK256Keccak),
    );

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
        AuthSingleSig::new(PublicKeyCommitment::from(EMPTY_WORD), AuthSchemeId::Falcon512Rpo),
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
        AuthSingleSig::new(PublicKeyCommitment::from(EMPTY_WORD), AuthSchemeId::EcdsaK256Keccak),
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
        accounts.into_iter().map(|(header, _)| header.to_commitment()).collect();
    let expected_commitments: BTreeSet<_> =
        expected_accounts.into_iter().map(|account| account.to_commitment()).collect();

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
        accounts.into_iter().map(|(header, _)| header.to_commitment()).collect();
    let expected_commitments: BTreeSet<_> =
        expected_accounts.into_iter().map(|account| account.to_commitment()).collect();

    assert_eq!(actual_commitments, expected_commitments);
}

#[tokio::test]
async fn prune_account_history_after_committed_transactions() {
    let (mut client, mock_rpc_api, keystore) = Box::pin(create_test_client()).await;

    // Create wallet and faucet
    let wallet = insert_new_wallet(&mut client, AccountStorageMode::Private, &keystore)
        .await
        .unwrap();
    let faucet = insert_new_fungible_faucet(&mut client, AccountStorageMode::Private, &keystore)
        .await
        .unwrap();
    let faucet_id = faucet.id();

    mock_rpc_api.prove_block();
    client.sync_state().await.unwrap();

    // Submit a mint tx (advances faucet nonce)
    let fungible_asset_1 = FungibleAsset::new(faucet_id, 100).unwrap();
    let tx_request_1 = TransactionRequestBuilder::new()
        .build_mint_fungible_asset(fungible_asset_1, wallet.id(), NoteType::Public, client.rng())
        .unwrap();
    Box::pin(client.submit_new_transaction(faucet_id, tx_request_1)).await.unwrap();

    mock_rpc_api.prove_block();
    client.sync_state().await.unwrap();

    // Further advances faucet nonce
    let fungible_asset_2 = FungibleAsset::new(faucet_id, 200).unwrap();
    let tx_request_2 = TransactionRequestBuilder::new()
        .build_mint_fungible_asset(fungible_asset_2, wallet.id(), NoteType::Public, client.rng())
        .unwrap();
    Box::pin(client.submit_new_transaction(faucet_id, tx_request_2)).await.unwrap();

    mock_rpc_api.prove_block();
    client.sync_state().await.unwrap();

    // At this point, the faucet has gone through nonces 0 → 1 → 2 (all committed).
    // Historical tables should have entries for each nonce.

    // Record faucet state before pruning
    let faucet_before = client.get_account(faucet_id).await.unwrap().unwrap();

    // Prune faucet history — should remove nonce 0 and 1, keep nonce 2
    let deleted = client.prune_account_history(faucet_id).await.unwrap();
    assert!(deleted > 0, "Should have pruned old committed states");

    // Verify: account is still fully readable and unchanged
    let faucet_after = client.get_account(faucet_id).await.unwrap().unwrap();
    assert_eq!(
        faucet_before.to_commitment(),
        faucet_after.to_commitment(),
        "Account state should be identical after pruning"
    );

    // Verify: can still read account headers
    let (header, _status) = client
        .get_account_headers()
        .await
        .unwrap()
        .into_iter()
        .find(|(h, _)| h.id() == faucet_id)
        .expect("Faucet should still appear in headers");
    assert_eq!(header.nonce().as_int(), 2, "Latest nonce should be 2");
}

#[tokio::test]
async fn prune_all_account_history_through_client() {
    let (mut client, mock_rpc_api, keystore) = Box::pin(create_test_client()).await;

    let wallet = insert_new_wallet(&mut client, AccountStorageMode::Private, &keystore)
        .await
        .unwrap();
    let faucet = insert_new_fungible_faucet(&mut client, AccountStorageMode::Private, &keystore)
        .await
        .unwrap();
    let faucet_id = faucet.id();

    mock_rpc_api.prove_block();
    client.sync_state().await.unwrap();

    // Mint and commit — creates historical entries for faucet
    let fungible_asset = FungibleAsset::new(faucet_id, 100).unwrap();
    let tx_request = TransactionRequestBuilder::new()
        .build_mint_fungible_asset(fungible_asset, wallet.id(), NoteType::Public, client.rng())
        .unwrap();
    Box::pin(client.submit_new_transaction(faucet_id, tx_request)).await.unwrap();

    mock_rpc_api.prove_block();
    client.sync_state().await.unwrap();

    let deleted = client.prune_all_account_history().await.unwrap();
    assert!(deleted > 0, "Should have pruned at least one old state");

    // Both accounts still readable
    assert!(client.get_account(wallet.id()).await.unwrap().is_some());
    assert!(client.get_account(faucet_id).await.unwrap().is_some());
}
