use std::vec::Vec;

use miden_lib::account::{auth::RpoFalcon512, faucets::BasicFungibleFaucet, wallets::BasicWallet};
use miden_objects::{
    Felt, Word,
    account::{Account, AccountBuilder, AccountStorageMode, AccountType, AuthSecretKey},
    asset::{FungibleAsset, TokenSymbol},
    crypto::dsa::rpo_falcon512::SecretKey,
    note::NoteType,
};
use rand::{RngCore, rngs::StdRng};

use crate::{
    Client, ClientError, keystore::FilesystemKeyStore, transaction::TransactionRequestBuilder,
    utils::execute_tx_and_consume_output_notes,
};

/// Builds a new wallet account and inserts it into the client and keystore.
pub async fn insert_new_wallet(
    client: &mut Client,
    storage_mode: AccountStorageMode,
    keystore: &FilesystemKeyStore<StdRng>,
) -> Result<(Account, Word, SecretKey), ClientError> {
    let mut init_seed = [0u8; 32];
    client.rng().fill_bytes(&mut init_seed);

    insert_new_wallet_with_seed(client, storage_mode, keystore, init_seed).await
}

/// Builds a new wallet account with the provided seed and inserts it into the client and keystore.
pub async fn insert_new_wallet_with_seed(
    client: &mut Client,
    storage_mode: AccountStorageMode,
    keystore: &FilesystemKeyStore<StdRng>,
    init_seed: [u8; 32],
) -> Result<(Account, Word, SecretKey), ClientError> {
    let key_pair = SecretKey::with_rng(client.rng());
    let pub_key = key_pair.public_key();

    keystore.add_key(&AuthSecretKey::RpoFalcon512(key_pair.clone()))?;

    let (account, seed) = AccountBuilder::new(init_seed)
        .account_type(AccountType::RegularAccountImmutableCode)
        .storage_mode(storage_mode)
        .with_auth_component(RpoFalcon512::new(pub_key))
        .with_component(BasicWallet)
        .build()?;

    client.add_account(&account, Some(seed), false).await?;

    Ok((account, seed, key_pair))
}

/// Builds a new fungible faucet account and inserts it into the client and keystore.
pub async fn insert_new_fungible_faucet(
    client: &mut Client,
    storage_mode: AccountStorageMode,
    keystore: &FilesystemKeyStore<StdRng>,
) -> Result<(Account, Word, SecretKey), ClientError> {
    let key_pair = SecretKey::with_rng(client.rng());
    let pub_key = key_pair.public_key();

    keystore.add_key(&AuthSecretKey::RpoFalcon512(key_pair.clone()))?;

    // we need to use an initial seed to create the wallet account
    let mut init_seed = [0u8; 32];
    client.rng().fill_bytes(&mut init_seed);

    let symbol = TokenSymbol::new("TEST").expect("Token symbol should be valid");
    let max_supply = Felt::try_from(9_999_999_u64.to_le_bytes().as_slice())
        .expect("u64 can be safely converted to a field element");

    let (account, seed) = AccountBuilder::new(init_seed)
        .account_type(AccountType::FungibleFaucet)
        .storage_mode(storage_mode)
        .with_auth_component(RpoFalcon512::new(pub_key))
        .with_component(
            BasicFungibleFaucet::new(symbol, 10, max_supply)
                .expect("Faucet component should be valid"),
        )
        .build()?;

    client.add_account(&account, Some(seed), false).await?;
    Ok((account, seed, key_pair))
}

/// Sets up a specified number of accounts and faucets, and mints tokens for each account.
///
/// This function creates a set of basic accounts and faucets, and mints tokens from each faucet to
/// the accounts based on the given balance matrix.
///
/// # Arguments
///
/// * `client` - The Miden client used to interact with the blockchain.
/// * `keystore` - The keystore used to securely store account keys.
/// * `num_accounts` - The number of accounts to create.
/// * `num_faucets` - The number of faucets to create.
/// * `balances` - A matrix where each row represents a faucet and each column a wallet. Every entry
///   represents the number of tokens to mint from a faucet to an wallet.
///
/// # Returns
///
/// Returns a tuple containing the created accounts and faucets as vectors.
pub async fn setup_accounts_and_faucets(
    client: &mut Client,
    keystore: &FilesystemKeyStore<StdRng>,
    storage_mode: AccountStorageMode,
    num_accounts: usize,
    num_faucets: usize,
    balances: Vec<Vec<u64>>,
) -> Result<(Vec<Account>, Vec<Account>), ClientError> {
    let mut accounts = Vec::with_capacity(num_accounts);
    for _ in 0..num_accounts {
        let (account, ..) = insert_new_wallet(client, storage_mode, keystore).await?;
        accounts.push(account);
    }

    let mut faucets = Vec::with_capacity(num_faucets);
    for _ in 0..num_faucets {
        let (faucet, ..) = insert_new_fungible_faucet(client, storage_mode, keystore).await?;
        faucets.push(faucet);
    }

    client.sync_state().await?;

    for (faucet_index, faucet) in faucets.iter().enumerate() {
        for (acct_index, account) in accounts.iter().enumerate() {
            let amount_to_mint = balances[faucet_index][acct_index];
            if amount_to_mint == 0 {
                continue;
            }

            let tx_request = TransactionRequestBuilder::new().build_mint_fungible_asset(
                FungibleAsset::new(faucet.id(), amount_to_mint)?,
                account.id(),
                NoteType::Public,
                client.rng(),
            )?;

            execute_tx_and_consume_output_notes(tx_request, client, faucet.id(), account.id())
                .await?;
        }
    }

    Ok((accounts, faucets))
}
