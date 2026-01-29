use assert_cmd::cargo::cargo_bin_cmd;
use predicates::str::contains;

// CLI ARGUMENT TESTS
// ================================================================================================

/// Tests that the help command works
#[test]
fn help_command() {
    let mut cmd = cargo_bin_cmd!("miden-bench");
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(contains("Benchmarks for the Miden client library"));
}

/// Tests that version command works
#[test]
fn version_command() {
    let mut cmd = cargo_bin_cmd!("miden-bench");
    cmd.arg("--version");
    cmd.assert().success().stdout(contains("miden-bench"));
}

/// Tests that invalid command fails gracefully
#[test]
fn invalid_command() {
    let mut cmd = cargo_bin_cmd!("miden-bench");
    cmd.arg("invalid-command");
    cmd.assert().failure();
}

/// Tests that the export subcommand help works
#[test]
fn export_help() {
    let mut cmd = cargo_bin_cmd!("miden-bench");
    cmd.args(["export", "--help"]);
    cmd.assert().success().stdout(contains("Benchmark account export/import"));
}

/// Tests that the sync subcommand help works
#[test]
fn sync_help() {
    let mut cmd = cargo_bin_cmd!("miden-bench");
    cmd.args(["sync", "--help"]);
    cmd.assert().success().stdout(contains("Benchmark sync operations"));
}

/// Tests that the transaction subcommand help works
#[test]
fn transaction_help() {
    let mut cmd = cargo_bin_cmd!("miden-bench");
    cmd.args(["transaction", "--help"]);
    cmd.assert().success().stdout(contains("Benchmark transaction operations"));
}

/// Tests that the deploy subcommand help works
#[test]
fn deploy_help() {
    let mut cmd = cargo_bin_cmd!("miden-bench");
    cmd.args(["deploy", "--help"]);
    cmd.assert().success().stdout(contains("Deploy a public wallet"));
}

// CLI OPTIONS TESTS
// ================================================================================================

/// Tests that size option is recognized (subcommand-specific)
#[test]
fn size_option_small() {
    let mut cmd = cargo_bin_cmd!("miden-bench");
    cmd.args(["export", "--size", "small", "--help"]);
    cmd.assert().success();
}

/// Tests that size option is recognized with all valid values
#[test]
fn size_option_all_values() {
    for size in &["small", "medium", "large", "very-large"] {
        let mut cmd = cargo_bin_cmd!("miden-bench");
        cmd.args(["export", "--size", size, "--help"]);
        cmd.assert().success();
    }
}

/// Tests that invalid size option fails
#[test]
fn size_option_invalid() {
    let mut cmd = cargo_bin_cmd!("miden-bench");
    cmd.args(["export", "--size", "invalid"]);
    cmd.assert().failure();
}

/// Tests that iterations option is recognized (global)
#[test]
fn iterations_option() {
    let mut cmd = cargo_bin_cmd!("miden-bench");
    cmd.args(["--iterations", "5", "export", "--help"]);
    cmd.assert().success();
}

/// Tests that network option is recognized with all valid values (global)
#[test]
fn network_option_all_values() {
    for network in &["localhost", "local", "devnet", "dev", "testnet", "test"] {
        let mut cmd = cargo_bin_cmd!("miden-bench");
        cmd.args(["--network", network, "sync", "--help"]);
        cmd.assert().success();
    }
}

/// Tests that custom network URL is accepted
#[test]
fn network_option_custom_url() {
    let mut cmd = cargo_bin_cmd!("miden-bench");
    cmd.args(["--network", "http://custom.node:8080", "sync", "--help"]);
    cmd.assert().success();
}

// OFFLINE BENCHMARK TESTS
// ================================================================================================

/// Tests that the export benchmark runs successfully with minimal settings
#[test]
fn export_benchmark_runs() {
    let mut cmd = cargo_bin_cmd!("miden-bench");
    cmd.args(["--iterations", "1", "export", "--size", "small"]);
    cmd.assert()
        .success()
        .stdout(contains("export/serialize_account"))
        .stdout(contains("export/deserialize_account"))
        .stdout(contains("export/roundtrip_account"));
}

// OUTPUT FORMAT TESTS
// ================================================================================================

/// Tests that output contains expected table format
#[test]
fn output_format() {
    let mut cmd = cargo_bin_cmd!("miden-bench");
    cmd.args(["--iterations", "1", "export", "--size", "small"]);
    cmd.assert()
        .success()
        // Table header with column names
        .stdout(contains("Export Benchmark"))
        .stdout(contains("Mean"))
        .stdout(contains("Min"))
        .stdout(contains("Max"))
        // Summary line
        .stdout(contains("Total benchmarks"));
}

// ACCOUNT SIZE TESTS
// ================================================================================================

/// Tests that different account sizes run successfully
#[test]
fn account_sizes_run() {
    // Run with small size (1 map * 10 entries = 10 total)
    let mut small_cmd = cargo_bin_cmd!("miden-bench");
    small_cmd.args(["--iterations", "1", "export", "--size", "small"]);
    small_cmd.assert().success().stdout(contains("10 entries"));

    // Run with medium size (2 maps * 100 entries = 200 total)
    let mut medium_cmd = cargo_bin_cmd!("miden-bench");
    medium_cmd.args(["--iterations", "1", "export", "--size", "medium"]);
    medium_cmd.assert().success().stdout(contains("100 entries"));
}

// MULTIPLE ITERATION TESTS
// ================================================================================================

/// Tests that multiple iterations produce statistics
#[test]
fn multiple_iterations_statistics() {
    let mut cmd = cargo_bin_cmd!("miden-bench");
    cmd.args(["--iterations", "3", "export", "--size", "small"]);
    cmd.assert()
        .success()
        // Should show statistics columns
        .stdout(contains("Mean"))
        .stdout(contains("Min"))
        .stdout(contains("Max"));
}
