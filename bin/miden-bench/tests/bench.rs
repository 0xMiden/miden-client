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

/// Tests that maps option is recognized (subcommand-specific)
#[test]
fn maps_option() {
    let mut cmd = cargo_bin_cmd!("miden-bench");
    cmd.args(["deploy", "--maps", "2", "--help"]);
    cmd.assert().success();
}

/// Tests that maps option works with a specific value
#[test]
fn maps_option_with_value() {
    let mut cmd = cargo_bin_cmd!("miden-bench");
    cmd.args(["deploy", "--maps", "5", "--help"]);
    cmd.assert().success();
}

/// Tests that invalid maps option fails
#[test]
fn maps_option_invalid() {
    let mut cmd = cargo_bin_cmd!("miden-bench");
    cmd.args(["deploy", "--maps", "invalid"]);
    cmd.assert().failure();
}

/// Tests that iterations option is recognized (transaction subcommand)
#[test]
fn iterations_option() {
    let mut cmd = cargo_bin_cmd!("miden-bench");
    cmd.args(["transaction", "--iterations", "5", "--help"]);
    cmd.assert().success();
}

/// Tests that network option is recognized with all valid values (global)
#[test]
fn network_option_all_values() {
    for network in &["localhost", "local", "devnet", "dev", "testnet", "test"] {
        let mut cmd = cargo_bin_cmd!("miden-bench");
        cmd.args(["--network", network, "deploy", "--help"]);
        cmd.assert().success();
    }
}

/// Tests that custom network URL is accepted
#[test]
fn network_option_custom_url() {
    let mut cmd = cargo_bin_cmd!("miden-bench");
    cmd.args(["--network", "http://custom.node:8080", "deploy", "--help"]);
    cmd.assert().success();
}
