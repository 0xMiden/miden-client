use std::io::Cursor;

use miden_protocol::account::AccountFile;
use miden_protocol::utils::{Deserializable, Serializable};

use crate::config::BenchConfig;
use crate::generators::{LargeAccountConfig, create_large_account};
use crate::metrics::{BenchmarkResult, run_benchmark_with_output};

/// Runs account export benchmarks
pub async fn run_export_benchmarks(config: &BenchConfig) -> anyhow::Result<Vec<BenchmarkResult>> {
    let mut results = Vec::new();

    let large_config = LargeAccountConfig::from_size(config.size);

    // Benchmark: Account serialization
    results.push(bench_account_serialization(config, &large_config).await?);

    // Benchmark: Account deserialization
    results.push(bench_account_deserialization(config, &large_config).await?);

    // Benchmark: Full export/import round-trip
    results.push(bench_account_roundtrip(config, &large_config).await?);

    Ok(results)
}

async fn bench_account_serialization(
    config: &BenchConfig,
    large_config: &LargeAccountConfig,
) -> anyhow::Result<BenchmarkResult> {
    let (account, secret_key) = create_large_account(large_config)?;
    let account_file = AccountFile::new(account, vec![secret_key]);

    let total_entries = large_config.num_map_slots * large_config.num_storage_map_entries;
    let name = format!("export/serialize_account ({total_entries} entries)");

    let result = run_benchmark_with_output(
        &name,
        config.iterations,
        || {
            let account_file = account_file.clone();
            async move {
                let mut buffer = Vec::new();
                account_file.write_into(&mut buffer);
                buffer
            }
        },
        Vec::len,
    )
    .await;

    Ok(result)
}

async fn bench_account_deserialization(
    config: &BenchConfig,
    large_config: &LargeAccountConfig,
) -> anyhow::Result<BenchmarkResult> {
    // First serialize to get the bytes
    let (account, secret_key) = create_large_account(large_config)?;
    let account_file = AccountFile::new(account, vec![secret_key]);
    let mut buffer = Vec::new();
    account_file.write_into(&mut buffer);
    let serialized_size = buffer.len();

    let total_entries = large_config.num_map_slots * large_config.num_storage_map_entries;
    let name = format!("export/deserialize_account ({total_entries} entries)");

    let mut result = run_benchmark_with_output(
        &name,
        config.iterations,
        || {
            let buffer = buffer.clone();
            async move {
                let mut cursor = Cursor::new(buffer);
                AccountFile::read_from(&mut cursor).expect("deserialization should succeed")
            }
        },
        |_| serialized_size,
    )
    .await;

    result = result.with_output_size(serialized_size);
    Ok(result)
}

async fn bench_account_roundtrip(
    config: &BenchConfig,
    large_config: &LargeAccountConfig,
) -> anyhow::Result<BenchmarkResult> {
    let (account, secret_key) = create_large_account(large_config)?;

    let total_entries = large_config.num_map_slots * large_config.num_storage_map_entries;
    let name = format!("export/roundtrip_account ({total_entries} entries)");

    let result = run_benchmark_with_output(
        &name,
        config.iterations,
        || {
            let account = account.clone();
            let secret_key = secret_key.clone();
            async move {
                // Serialize
                let account_file = AccountFile::new(account, vec![secret_key]);
                let mut buffer = Vec::new();
                account_file.write_into(&mut buffer);

                // Deserialize
                let mut cursor = Cursor::new(&buffer);
                let _ =
                    AccountFile::read_from(&mut cursor).expect("deserialization should succeed");

                buffer
            }
        },
        Vec::len,
    )
    .await;

    Ok(result)
}
