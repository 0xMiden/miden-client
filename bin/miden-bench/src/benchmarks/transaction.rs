use std::path::PathBuf;
use std::sync::Arc;

use miden_client::account::component::{AccountComponent, BasicFungibleFaucet, BasicWallet};
use miden_client::account::{AccountBuilder, AccountId, AccountStorageMode, AccountType};
use miden_client::asset::{FungibleAsset, TokenSymbol};
use miden_client::builder::ClientBuilder;
use miden_client::crypto::RpoRandomCoin;
use miden_client::keystore::FilesystemKeyStore;
use miden_client::note::{NoteAttachment, create_p2id_note};
use miden_client::rpc::GrpcClient;
use miden_client::transaction::{OutputNote, TransactionRequestBuilder};
use miden_client::{DebugMode, Felt};
use miden_client_sqlite_store::ClientBuilderSqliteExt;
use miden_protocol::account::auth::AuthSecretKey;
use miden_standards::account::auth::AuthFalcon512Rpo;
use rand::Rng;

use crate::AccountSize;
use crate::config::BenchConfig;
use crate::metrics::{BenchmarkResult, measure_time_async};
use crate::spinner::with_spinner;

// Helper to create a unique temp directory for each benchmark run
fn create_temp_dir(config: &BenchConfig, suffix: &str) -> PathBuf {
    let base = config.temp_dir();
    let unique_id = uuid::Uuid::new_v4();
    let path = base.join(format!("miden-bench-{suffix}-{unique_id}"));
    std::fs::create_dir_all(&path).expect("Failed to create temp directory");
    path
}

// Helper to create a client for benchmarking
async fn create_benchmark_client(
    config: &BenchConfig,
    suffix: &str,
) -> anyhow::Result<(miden_client::Client<FilesystemKeyStore>, FilesystemKeyStore, PathBuf)> {
    let temp_dir = create_temp_dir(config, suffix);
    let store_path = temp_dir.join("store.sqlite3");
    let keystore_path = temp_dir.join("keystore");
    std::fs::create_dir_all(&keystore_path)?;

    let mut rng = rand::rng();
    let coin_seed: [u64; 4] = rng.random();
    let rng = RpoRandomCoin::new(coin_seed.map(Felt::new).into());

    let keystore = FilesystemKeyStore::new(keystore_path.clone())
        .expect("Failed to create filesystem keystore");

    let client = ClientBuilder::new()
        .rpc(Arc::new(GrpcClient::new(&config.network, 30_000)))
        .rng(Box::new(rng))
        .sqlite_store(store_path)
        .filesystem_keystore(keystore_path.to_str().expect("keystore path should be valid UTF-8"))
        .in_debug_mode(DebugMode::Disabled)
        .tx_graceful_blocks(None)
        .build()
        .await?;

    Ok((client, keystore, temp_dir))
}

/// Returns the number of notes to generate based on account size
fn num_notes_for_size(size: AccountSize) -> usize {
    match size {
        AccountSize::Small => 5,
        AccountSize::Medium => 50,
        AccountSize::Large => 100,
        AccountSize::VeryLarge => 1000,
    }
}

/// Runs transaction benchmarks (requires a running node)
pub async fn run_transaction_benchmarks(
    config: &BenchConfig,
) -> anyhow::Result<Vec<BenchmarkResult>> {
    let mut results = Vec::new();

    let num_notes = num_notes_for_size(config.size);

    // First, try to connect to the node
    println!("Connecting to node at {}...", config.network);

    let (mut client, keystore, _temp_dir) = match create_benchmark_client(config, "tx-init").await {
        Ok(result) => result,
        Err(e) => {
            println!("Failed to connect to node: {e}");
            println!("Skipping transaction benchmarks (requires a running Miden node).");
            results
                .push(BenchmarkResult::new("transaction/connection_failed").with_metadata(
                    format!("Could not connect to node at {}: {e}", config.network),
                ));
            return Ok(results);
        },
    };

    // Sync with the network first
    if let Err(e) = client.sync_state().await {
        println!("Failed to sync with node: {e}");
        println!("Skipping transaction benchmarks.");
        results.push(
            BenchmarkResult::new("transaction/sync_failed")
                .with_metadata(format!("Failed to sync: {e}")),
        );
        return Ok(results);
    }

    let chain_height = client.get_sync_height().await?;
    println!("Connected successfully. Chain height: {chain_height}");
    println!("Output notes per transaction: {num_notes}");

    // Benchmark 1: Transaction execution time (without proving)
    let execution_result = with_spinner("Benchmarking transaction execution", || {
        benchmark_tx_execution(config, num_notes)
    })
    .await?;
    results.push(execution_result);

    // Benchmark 2: Transaction proving time
    let proving_result = with_spinner("Benchmarking transaction proving", || {
        benchmark_tx_proving(config, num_notes)
    })
    .await?;
    results.push(proving_result);

    // Benchmark 3: Full transaction (execute + prove + submit)
    let full_result = with_spinner("Benchmarking full transaction", || {
        Box::pin(benchmark_tx_full(config, &keystore, num_notes))
    })
    .await?;
    results.push(full_result);

    Ok(results)
}

/// Benchmarks transaction execution time
async fn benchmark_tx_execution(
    config: &BenchConfig,
    num_notes: usize,
) -> anyhow::Result<BenchmarkResult> {
    let bench_name = format!("execute ({num_notes} notes)");

    let mut result = BenchmarkResult::new(&bench_name);

    for i in 0..config.iterations {
        let (mut client, keystore, _) =
            create_benchmark_client(config, &format!("tx-exec-iter-{i}")).await?;
        client.sync_state().await?;

        // Create a faucet and recipient accounts
        let (faucet, _) = create_faucet(&mut client, &keystore).await?;
        let recipients = create_recipient_accounts(&mut client, &keystore, num_notes).await?;

        // Create a transaction that mints notes to all recipients
        let output_notes = create_mint_notes(&mut client, faucet.id(), &recipients, 100)?;

        let tx_request = TransactionRequestBuilder::new().own_output_notes(output_notes).build()?;

        // Measure execution time only
        let (_, duration) = measure_time_async(|| async {
            client.execute_transaction(faucet.id(), tx_request).await
        })
        .await;

        result.add_iteration(duration);
    }

    result = result
        .with_metadata(format!("Transaction execution (no proving), {num_notes} output notes"));

    Ok(result)
}

/// Benchmarks transaction proving time
async fn benchmark_tx_proving(
    config: &BenchConfig,
    num_notes: usize,
) -> anyhow::Result<BenchmarkResult> {
    let bench_name = format!("prove ({num_notes} notes)");

    let mut result = BenchmarkResult::new(&bench_name);

    for i in 0..config.iterations {
        let (mut client, keystore, _) =
            create_benchmark_client(config, &format!("tx-prove-iter-{i}")).await?;
        client.sync_state().await?;

        let (faucet, _) = create_faucet(&mut client, &keystore).await?;
        let recipients = create_recipient_accounts(&mut client, &keystore, num_notes).await?;

        let output_notes = create_mint_notes(&mut client, faucet.id(), &recipients, 100)?;

        let tx_request = TransactionRequestBuilder::new().own_output_notes(output_notes).build()?;

        // Execute first (not measured)
        let tx_result = client.execute_transaction(faucet.id(), tx_request).await?;

        // Measure proving time only
        let (proven_tx, duration) =
            measure_time_async(|| async { client.prove_transaction(&tx_result).await }).await;

        if let Ok(proven) = proven_tx {
            result.add_iteration(duration);
            // Record proof size
            let proof_bytes = proven.proof().to_bytes();
            result = result.with_output_size(proof_bytes.len());
        } else {
            // If proving fails, still record the time
            result.add_iteration(duration);
        }
    }

    result = result.with_metadata(format!("Transaction proving, {num_notes} output notes"));

    Ok(result)
}

/// Benchmarks full transaction (execute + prove + submit)
async fn benchmark_tx_full(
    config: &BenchConfig,
    _parent_keystore: &FilesystemKeyStore,
    num_notes: usize,
) -> anyhow::Result<BenchmarkResult> {
    let bench_name = format!("full ({num_notes} notes)");

    let mut result = BenchmarkResult::new(&bench_name);

    for i in 0..config.iterations {
        let (mut client, keystore, _) =
            create_benchmark_client(config, &format!("tx-full-iter-{i}")).await?;
        client.sync_state().await?;

        let (faucet, _) = create_faucet(&mut client, &keystore).await?;
        let recipients = create_recipient_accounts(&mut client, &keystore, num_notes).await?;

        let output_notes = create_mint_notes(&mut client, faucet.id(), &recipients, 100)?;

        let tx_request = TransactionRequestBuilder::new().own_output_notes(output_notes).build()?;

        // Measure full transaction time (execute + prove + submit)
        let (_, duration) = measure_time_async(|| async {
            client.submit_new_transaction(faucet.id(), tx_request).await
        })
        .await;

        result.add_iteration(duration);
    }

    result = result.with_metadata(format!(
        "Full transaction (execute + prove + submit), {num_notes} output notes"
    ));

    Ok(result)
}

// HELPERS
// ================================================================================================

/// Creates a new fungible faucet account
async fn create_faucet(
    client: &mut miden_client::Client<FilesystemKeyStore>,
    keystore: &FilesystemKeyStore,
) -> anyhow::Result<(miden_protocol::account::Account, AuthSecretKey)> {
    let mut rng = rand::rng();
    let mut init_seed = [0u8; 32];
    rng.fill(&mut init_seed);

    let key_pair = AuthSecretKey::new_falcon512_rpo();
    let auth_component: AccountComponent =
        AuthFalcon512Rpo::new(key_pair.public_key().to_commitment()).into();

    let symbol = TokenSymbol::new("BNCH")?;
    let max_supply = Felt::try_from(9_999_999_u64.to_le_bytes().as_slice())?;

    let account = AccountBuilder::new(init_seed)
        .account_type(AccountType::FungibleFaucet)
        .storage_mode(AccountStorageMode::Private)
        .with_auth_component(auth_component)
        .with_component(BasicFungibleFaucet::new(symbol, 10, max_supply)?)
        .build()?;

    keystore.add_key(&key_pair)?;
    client.add_account(&account, false).await?;

    Ok((account, key_pair))
}

/// Creates multiple recipient wallet accounts
async fn create_recipient_accounts(
    client: &mut miden_client::Client<FilesystemKeyStore>,
    keystore: &FilesystemKeyStore,
    count: usize,
) -> anyhow::Result<Vec<AccountId>> {
    let mut recipients = Vec::with_capacity(count);

    for _ in 0..count {
        let mut rng = rand::rng();
        let mut init_seed = [0u8; 32];
        rng.fill(&mut init_seed);

        let key_pair = AuthSecretKey::new_falcon512_rpo();
        let auth_component: AccountComponent =
            AuthFalcon512Rpo::new(key_pair.public_key().to_commitment()).into();

        let account = AccountBuilder::new(init_seed)
            .account_type(AccountType::RegularAccountImmutableCode)
            .storage_mode(AccountStorageMode::Private)
            .with_auth_component(auth_component)
            .with_component(BasicWallet)
            .build()?;

        keystore.add_key(&key_pair)?;
        client.add_account(&account, false).await?;

        recipients.push(account.id());
    }

    Ok(recipients)
}

/// Creates mint notes (P2ID notes with fungible assets) for multiple recipients
fn create_mint_notes(
    client: &mut miden_client::Client<FilesystemKeyStore>,
    faucet_id: AccountId,
    recipients: &[AccountId],
    amount_per_note: u64,
) -> anyhow::Result<Vec<OutputNote>> {
    let mut notes = Vec::with_capacity(recipients.len());

    for &recipient_id in recipients {
        let asset = FungibleAsset::new(faucet_id, amount_per_note)?;

        let note = create_p2id_note(
            faucet_id,
            recipient_id,
            vec![asset.into()],
            miden_client::note::NoteType::Private,
            NoteAttachment::default(),
            client.rng(),
        )?;

        notes.push(OutputNote::Full(note));
    }

    Ok(notes)
}
