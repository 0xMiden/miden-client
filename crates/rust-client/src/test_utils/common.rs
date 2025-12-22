use std::boxed::Box;
use std::env::temp_dir;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::println;
use std::string::ToString;
use std::time::{Duration, Instant};
use std::vec::Vec;

use anyhow::{Context, Result};
use miden_objects::account::auth::AuthSecretKey;
use miden_objects::account::{Account, AccountId, AccountStorageMode};
use miden_objects::asset::{Asset, FungibleAsset, TokenSymbol};
use miden_objects::note::{NoteId, NoteType};
use miden_objects::transaction::{OutputNote, TransactionId};
use miden_objects::{Felt, FieldElement};
use rand::RngCore;
use uuid::Uuid;

use crate::account::component::{
    AccountComponent,
    AuthEcdsaK256Keccak,
    AuthRpoFalcon512,
    BasicFungibleFaucet,
    BasicWallet,
};
use crate::account::{AccountBuilder, AccountType, StorageSlot};
use crate::auth::AuthSchemeId;
use crate::crypto::FeltRng;
pub use crate::keystore::FilesystemKeyStore;
use crate::note::{Note, create_p2id_note};
use crate::rpc::RpcError;
use crate::store::{NoteFilter, TransactionFilter};
use crate::sync::SyncSummary;
use crate::testing::account_id::ACCOUNT_ID_REGULAR_PRIVATE_ACCOUNT_UPDATABLE_CODE;
use crate::transaction::{
    NoteArgs,
    TransactionRequest,
    TransactionRequestBuilder,
    TransactionRequestError,
    TransactionStatus,
};
use crate::{Client, ClientError};

pub type TestClient = Client<FilesystemKeyStore>;

// CONSTANTS
// ================================================================================================
pub const ACCOUNT_ID_REGULAR: u128 = ACCOUNT_ID_REGULAR_PRIVATE_ACCOUNT_UPDATABLE_CODE;

/// Constant that represents the number of blocks until the p2id can be recalled. If this value is
/// too low, some tests might fail due to expected recall failures not happening.
pub const RECALL_HEIGHT_DELTA: u32 = 50;

pub fn create_test_store_path() -> PathBuf {
    let mut temp_file = temp_dir();
    temp_file.push(format!("{}.sqlite3", Uuid::new_v4()));
    temp_file
}

/// Inserts a new wallet account into the client and into the keystore.
pub async fn insert_new_wallet(
    client: &mut TestClient,
    storage_mode: AccountStorageMode,
    keystore: &FilesystemKeyStore,
    auth_scheme: AuthSchemeId,
) -> Result<(Account, AuthSecretKey), ClientError> {
    let mut init_seed = [0u8; 32];
    client.rng().fill_bytes(&mut init_seed);

    insert_new_wallet_with_seed(client, storage_mode, keystore, init_seed, auth_scheme).await
}

/// Inserts a new wallet account built with the provided seed into the client and into the keystore.
pub async fn insert_new_wallet_with_seed(
    client: &mut TestClient,
    storage_mode: AccountStorageMode,
    keystore: &FilesystemKeyStore,
    init_seed: [u8; 32],
    auth_scheme: AuthSchemeId,
) -> Result<(Account, AuthSecretKey), ClientError> {
    let (key_pair, auth_component) = match auth_scheme {
        AuthSchemeId::RpoFalcon512 => {
            let key_pair = AuthSecretKey::new_rpo_falcon512();
            let auth_component: AccountComponent =
                AuthRpoFalcon512::new(key_pair.public_key().to_commitment()).into();
            (key_pair, auth_component)
        },
        AuthSchemeId::EcdsaK256Keccak => {
            let key_pair = AuthSecretKey::new_ecdsa_k256_keccak();
            let auth_component: AccountComponent =
                AuthEcdsaK256Keccak::new(key_pair.public_key().to_commitment()).into();
            (key_pair, auth_component)
        },
        scheme => {
            return Err(ClientError::TransactionRequestError(
                TransactionRequestError::UnsupportedAuthSchemeId(scheme.as_u8()),
            ));
        },
    };

    keystore.add_key(&key_pair).unwrap();

    let account = AccountBuilder::new(init_seed)
        .account_type(AccountType::RegularAccountImmutableCode)
        .storage_mode(storage_mode)
        .with_auth_component(auth_component)
        .with_component(BasicWallet)
        .build()
        .unwrap();

    client.add_account(&account, false).await?;

    Ok((account, key_pair))
}

/// Inserts a new fungible faucet account into the client and into the keystore.
pub async fn insert_new_fungible_faucet(
    client: &mut TestClient,
    storage_mode: AccountStorageMode,
    keystore: &FilesystemKeyStore,
    auth_scheme: AuthSchemeId,
) -> Result<(Account, AuthSecretKey), ClientError> {
    let (key_pair, auth_component) = match auth_scheme {
        AuthSchemeId::RpoFalcon512 => {
            let key_pair = AuthSecretKey::new_rpo_falcon512();
            let auth_component: AccountComponent =
                AuthRpoFalcon512::new(key_pair.public_key().to_commitment()).into();
            (key_pair, auth_component)
        },
        AuthSchemeId::EcdsaK256Keccak => {
            let key_pair = AuthSecretKey::new_ecdsa_k256_keccak();
            let auth_component: AccountComponent =
                AuthEcdsaK256Keccak::new(key_pair.public_key().to_commitment()).into();
            (key_pair, auth_component)
        },
        scheme => {
            return Err(ClientError::TransactionRequestError(
                TransactionRequestError::UnsupportedAuthSchemeId(scheme.as_u8()),
            ));
        },
    };

    keystore.add_key(&key_pair).unwrap();

    // we need to use an initial seed to create the faucet account
    let mut init_seed = [0u8; 32];
    client.rng().fill_bytes(&mut init_seed);

    let symbol = TokenSymbol::new("TEST").unwrap();
    let max_supply = Felt::try_from(9_999_999_u64.to_le_bytes().as_slice())
        .expect("u64 can be safely converted to a field element");

    let account = AccountBuilder::new(init_seed)
        .account_type(AccountType::FungibleFaucet)
        .storage_mode(storage_mode)
        .with_auth_component(auth_component)
        .with_component(BasicFungibleFaucet::new(symbol, 10, max_supply).unwrap())
        .build()
        .unwrap();

    client.add_account(&account, false).await?;
    Ok((account, key_pair))
}

/// Executes a transaction and asserts that it fails with the expected error.
pub async fn execute_failing_tx(
    client: &mut TestClient,
    account_id: AccountId,
    tx_request: TransactionRequest,
    expected_error: ClientError,
) {
    println!("Executing transaction...");
    // We compare string since we can't compare the error directly
    assert_eq!(
        Box::pin(client.submit_new_transaction(account_id, tx_request))
            .await
            .unwrap_err()
            .to_string(),
        expected_error.to_string()
    );
}

/// Executes a transaction and waits for it to be committed.
pub async fn execute_tx_and_sync(
    client: &mut TestClient,
    account_id: AccountId,
    tx_request: TransactionRequest,
) -> Result<()> {
    let transaction_id = Box::pin(client.submit_new_transaction(account_id, tx_request)).await?;
    wait_for_tx(client, transaction_id).await?;
    Ok(())
}

/// Syncs the client and waits for the transaction to be committed.
pub async fn wait_for_tx(client: &mut TestClient, transaction_id: TransactionId) -> Result<()> {
    // wait until tx is committed
    let now = Instant::now();
    println!("Syncing State...");
    loop {
        client
            .sync_state()
            .await
            .with_context(|| "failed to sync client state while waiting for transaction")?;

        // Check if executed transaction got committed by the node
        let tracked_transaction = client
            .get_transactions(TransactionFilter::Ids(vec![transaction_id]))
            .await
            .with_context(|| format!("failed to get transaction with ID: {transaction_id}"))?
            .pop()
            .with_context(|| format!("transaction with ID {transaction_id} not found"))?;

        match tracked_transaction.status {
            TransactionStatus::Committed { block_number, .. } => {
                println!("tx committed in {block_number}");
                break;
            },
            TransactionStatus::Pending => {
                std::thread::sleep(Duration::from_secs(1));
            },
            TransactionStatus::Discarded(cause) => {
                anyhow::bail!("transaction was discarded with cause: {cause:?}");
            },
        }

        // Log wait time in a file if the env var is set
        // This allows us to aggregate and measure how long the tests are waiting for transactions
        // to be committed
        if std::env::var("LOG_WAIT_TIMES") == Ok("true".to_string()) {
            let elapsed = now.elapsed();
            let wait_times_dir = std::path::PathBuf::from("wait_times");
            std::fs::create_dir_all(&wait_times_dir)
                .with_context(|| "failed to create wait_times directory")?;

            let elapsed_time_file = wait_times_dir.join(format!("wait_time_{}", Uuid::new_v4()));
            let mut file = OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(elapsed_time_file)
                .with_context(|| "failed to create elapsed time file")?;
            writeln!(file, "{:?}", elapsed.as_millis())
                .with_context(|| "failed to write elapsed time to file")?;
        }
    }
    Ok(())
}

/// Syncs until `amount_of_blocks` have been created onchain compared to client's sync height
pub async fn wait_for_blocks(client: &mut TestClient, amount_of_blocks: u32) -> SyncSummary {
    let current_block = client.get_sync_height().await.unwrap();
    let final_block = current_block + amount_of_blocks;
    println!("Syncing until block {final_block}...",);
    loop {
        let summary = client.sync_state().await.unwrap();
        println!("Synced to block {} (syncing until {})...", summary.block_num, final_block);

        if summary.block_num >= final_block {
            return summary;
        }

        std::thread::sleep(Duration::from_secs(3));
    }
}

/// Idles until `amount_of_blocks` have been created onchain compared to client's sync height
/// without advancing the client's sync height
pub async fn wait_for_blocks_no_sync(client: &mut TestClient, amount_of_blocks: u32) {
    let current_block = client.get_sync_height().await.unwrap();
    let final_block = current_block + amount_of_blocks;
    println!("Waiting until block {final_block}...",);
    loop {
        let (latest_block, _) =
            client.test_rpc_api().get_block_header_by_number(None, false).await.unwrap();
        println!(
            "Waited up to block {} (waiting until {})...",
            latest_block.block_num(),
            final_block
        );

        if latest_block.block_num() >= final_block {
            return;
        }

        std::thread::sleep(Duration::from_secs(3));
    }
}

/// Waits for node to be running.
///
/// # Panics
///
/// This function will panic if it does `NUMBER_OF_NODE_ATTEMPTS` unsuccessful checks or if we
/// receive an error other than a connection related error.
pub async fn wait_for_node(client: &mut TestClient) {
    const NODE_TIME_BETWEEN_ATTEMPTS: u64 = 5;
    const NUMBER_OF_NODE_ATTEMPTS: u64 = 60;

    println!(
        "Waiting for Node to be up. Checking every {NODE_TIME_BETWEEN_ATTEMPTS}s for {NUMBER_OF_NODE_ATTEMPTS} tries..."
    );

    for _try_number in 0..NUMBER_OF_NODE_ATTEMPTS {
        match client.sync_state().await {
            Err(ClientError::RpcError(RpcError::ConnectionError(_))) => {
                std::thread::sleep(Duration::from_secs(NODE_TIME_BETWEEN_ATTEMPTS));
            },
            Err(other_error) => {
                panic!("Unexpected error: {other_error}");
            },
            _ => return,
        }
    }

    panic!("Unable to connect to node");
}

pub const MINT_AMOUNT: u64 = 1000;
pub const TRANSFER_AMOUNT: u64 = 59;

/// Sets up a basic client and returns two basic accounts and a faucet account (in that order).
pub async fn setup_two_wallets_and_faucet(
    client: &mut TestClient,
    accounts_storage_mode: AccountStorageMode,
    keystore: &FilesystemKeyStore,
    auth_scheme: AuthSchemeId,
) -> Result<(Account, Account, Account)> {
    // Ensure clean state
    let account_headers = client
        .get_account_headers()
        .await
        .with_context(|| "failed to get account headers")?;
    anyhow::ensure!(account_headers.is_empty(), "Expected empty account headers for clean state");

    let transactions = client
        .get_transactions(TransactionFilter::All)
        .await
        .with_context(|| "failed to get transactions")?;
    anyhow::ensure!(transactions.is_empty(), "Expected empty transactions for clean state");

    let input_notes = client
        .get_input_notes(NoteFilter::All)
        .await
        .with_context(|| "failed to get input notes")?;
    anyhow::ensure!(input_notes.is_empty(), "Expected empty input notes for clean state");

    // Create faucet account
    let (faucet_account, _) =
        insert_new_fungible_faucet(client, accounts_storage_mode, keystore, auth_scheme)
            .await
            .with_context(|| "failed to insert new fungible faucet account")?;

    // Create regular accounts
    let (first_basic_account, ..) =
        insert_new_wallet(client, accounts_storage_mode, keystore, auth_scheme)
            .await
            .with_context(|| "failed to insert first basic wallet account")?;

    let (second_basic_account, ..) =
        insert_new_wallet(client, accounts_storage_mode, keystore, auth_scheme)
            .await
            .with_context(|| "failed to insert second basic wallet account")?;

    println!("Syncing State...");
    client.sync_state().await.with_context(|| "failed to sync client state")?;

    // Get Faucet and regular accounts
    println!("Fetching Accounts...");
    Ok((first_basic_account, second_basic_account, faucet_account))
}

/// Sets up a basic client and returns a basic account and a faucet account.
pub async fn setup_wallet_and_faucet(
    client: &mut TestClient,
    accounts_storage_mode: AccountStorageMode,
    keystore: &FilesystemKeyStore,
    auth_scheme: AuthSchemeId,
) -> Result<(Account, Account)> {
    let (faucet_account, _) =
        insert_new_fungible_faucet(client, accounts_storage_mode, keystore, auth_scheme)
            .await
            .with_context(|| "failed to insert new fungible faucet account")?;

    let (basic_account, ..) =
        insert_new_wallet(client, accounts_storage_mode, keystore, auth_scheme)
            .await
            .with_context(|| "failed to insert new wallet account")?;

    Ok((basic_account, faucet_account))
}

/// Mints a note from `faucet_account_id` for `basic_account_id` and returns the executed
/// transaction ID and the note with [`MINT_AMOUNT`] units of the corresponding fungible asset.
pub async fn mint_note(
    client: &mut TestClient,
    basic_account_id: AccountId,
    faucet_account_id: AccountId,
    note_type: NoteType,
) -> (TransactionId, Note) {
    // Create a Mint Tx for MINT_AMOUNT units of our fungible asset
    let fungible_asset = FungibleAsset::new(faucet_account_id, MINT_AMOUNT).unwrap();
    println!("Minting Asset");
    let tx_request = TransactionRequestBuilder::new()
        .build_mint_fungible_asset(fungible_asset, basic_account_id, note_type, client.rng())
        .unwrap();
    let tx_id =
        Box::pin(client.submit_new_transaction(fungible_asset.faucet_id(), tx_request.clone()))
            .await
            .unwrap();

    // Check that note is committed and return it
    println!("Fetching Committed Notes...");
    (tx_id, tx_request.expected_output_own_notes().pop().unwrap())
}

/// Executes a transaction that consumes the provided notes and returns the transaction ID.
/// This assumes the notes contain assets.
pub async fn consume_notes(
    client: &mut TestClient,
    account_id: AccountId,
    input_notes: &[Note],
) -> TransactionId {
    println!("Consuming Note...");
    let tx_request = TransactionRequestBuilder::new()
        .build_consume_notes(input_notes.iter().map(Note::id).collect())
        .unwrap();
    Box::pin(client.submit_new_transaction(account_id, tx_request)).await.unwrap()
}

/// Asserts that the account has a single asset with the expected amount.
pub async fn assert_account_has_single_asset(
    client: &TestClient,
    account_id: AccountId,
    asset_account_id: AccountId,
    expected_amount: u64,
) {
    let regular_account: Account =
        client.get_account(account_id).await.unwrap().unwrap().try_into().unwrap();

    assert_eq!(regular_account.vault().assets().count(), 1);
    let asset = regular_account.vault().assets().next().unwrap();

    if let Asset::Fungible(fungible_asset) = asset {
        assert_eq!(fungible_asset.faucet_id(), asset_account_id);
        assert_eq!(fungible_asset.amount(), expected_amount);
    } else {
        panic!("Account has consumed a note and should have a fungible asset");
    }
}

/// Tries to consume the note and asserts that the expected error is returned.
pub async fn assert_note_cannot_be_consumed_twice(
    client: &mut TestClient,
    consuming_account_id: AccountId,
    note_to_consume_id: NoteId,
) {
    // Check that we can't consume the P2ID note again
    println!("Consuming Note...");

    // Double-spend error expected to be received since we are consuming the same note
    let tx_request = TransactionRequestBuilder::new()
        .build_consume_notes(vec![note_to_consume_id])
        .unwrap();

    match Box::pin(client.submit_new_transaction(consuming_account_id, tx_request)).await {
        Err(ClientError::TransactionRequestError(
            TransactionRequestError::InputNoteAlreadyConsumed(_),
        )) => {},
        Ok(_) => panic!("Double-spend error: Note should not be consumable!"),
        err => panic!("Unexpected error {:?} for note ID: {}", err, note_to_consume_id.to_hex()),
    }
}

/// Creates a transaction request that mints assets for each `target_id` account.
pub fn mint_multiple_fungible_asset(
    asset: FungibleAsset,
    target_id: &[AccountId],
    note_type: NoteType,
    rng: &mut impl FeltRng,
) -> TransactionRequest {
    let notes = target_id
        .iter()
        .map(|account_id| {
            OutputNote::Full(
                create_p2id_note(
                    asset.faucet_id(),
                    *account_id,
                    vec![asset.into()],
                    note_type,
                    Felt::ZERO,
                    rng,
                )
                .unwrap(),
            )
        })
        .collect::<Vec<OutputNote>>();

    TransactionRequestBuilder::new().own_output_notes(notes).build().unwrap()
}

/// Executes a transaction and consumes the resulting unauthenticated notes immediately without
/// waiting for the first transaction to be committed.
pub async fn execute_tx_and_consume_output_notes(
    tx_request: TransactionRequest,
    client: &mut TestClient,
    executor: AccountId,
    consumer: AccountId,
) -> TransactionId {
    let output_notes = tx_request
        .expected_output_own_notes()
        .into_iter()
        .map(|note| (note, None::<NoteArgs>))
        .collect::<Vec<(Note, Option<NoteArgs>)>>();

    Box::pin(client.submit_new_transaction(executor, tx_request)).await.unwrap();

    let tx_request = TransactionRequestBuilder::new()
        .unauthenticated_input_notes(output_notes)
        .build()
        .unwrap();
    Box::pin(client.submit_new_transaction(consumer, tx_request)).await.unwrap()
}

/// Mints assets for the target account and consumes them immediately without waiting for the first
/// transaction to be committed.
pub async fn mint_and_consume(
    client: &mut TestClient,
    basic_account_id: AccountId,
    faucet_account_id: AccountId,
    note_type: NoteType,
) -> TransactionId {
    let tx_request = TransactionRequestBuilder::new()
        .build_mint_fungible_asset(
            FungibleAsset::new(faucet_account_id, MINT_AMOUNT).unwrap(),
            basic_account_id,
            note_type,
            client.rng(),
        )
        .unwrap();

    Box::pin(execute_tx_and_consume_output_notes(
        tx_request,
        client,
        faucet_account_id,
        basic_account_id,
    ))
    .await
}

/// Creates and inserts an account with custom code as a component into the client.
pub async fn insert_account_with_custom_component(
    client: &mut TestClient,
    custom_code: &str,
    storage_slots: Vec<StorageSlot>,
    storage_mode: AccountStorageMode,
    keystore: &FilesystemKeyStore,
) -> Result<(Account, AuthSecretKey), ClientError> {
    let component_code = miden_lib::utils::CodeBuilder::default()
        .compile_component_code("custom::component", custom_code)
        .map_err(|err| ClientError::TransactionRequestError(err.into()))?;
    let custom_component = AccountComponent::new(component_code, storage_slots)
        .map_err(ClientError::AccountError)?
        .with_supports_all_types();

    let mut init_seed = [0u8; 32];
    client.rng().fill_bytes(&mut init_seed);

    let key_pair = AuthSecretKey::new_rpo_falcon512_with_rng(client.rng());
    let pub_key = key_pair.public_key();
    keystore.add_key(&key_pair).unwrap();

    let account = AccountBuilder::new(init_seed)
        .account_type(AccountType::RegularAccountImmutableCode)
        .storage_mode(storage_mode)
        .with_auth_component(AuthRpoFalcon512::new(pub_key.to_commitment()))
        .with_component(BasicWallet)
        .with_component(custom_component)
        .build()
        .map_err(ClientError::AccountError)?;

    client.add_account(&account, false).await?;

    Ok((account, key_pair))
}
