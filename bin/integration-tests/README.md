# Miden Client Integration Tests

This directory contains integration tests for the Miden client library. These tests verify the functionality of the client against a running Miden node.

## Features

- **Parallel Execution**: Run tests in parallel to significantly reduce total execution time
- **Test Filtering**: Filter tests by name patterns, categories, or exclude specific tests
- **Flexible Configuration**: Configurable RPC endpoints, timeouts, and parallel job counts
- **Comprehensive Reporting**: Detailed test results with timing statistics and progress tracking
- **cargo-nextest-like Experience**: Similar filtering and execution patterns as cargo-nextest

## Installation

To install the integration tests binary:

```bash
make install-tests
```

This will build and install the `miden-client-integration-tests` binary to your system.

## Usage

### Running the Binary

The integration tests binary can be run with various command-line options:

```bash
miden-client-integration-tests [OPTIONS]
```

### Command-Line Options

- `-n, --network <NETWORK>` - The network to use. Options are `devnet`, `testnet`, `localhost` or a custom RPC endpoint (default: `localhost`)
- `-t, --timeout <MILLISECONDS>` - Timeout for RPC requests in milliseconds (default: `10000`)
- `-j, --jobs <NUMBER>` - Number of tests to run in parallel (default: auto-detected CPU cores, set to `1` for sequential execution)
- `-f, --filter <REGEX>` - Filter tests by name using regex patterns
- `--contains <STRING>` - Only run tests whose names contain this substring
- `--exclude <REGEX>` - Exclude tests whose names match this regex pattern
- `--retry-count <NUMBER>` - Number of times to retry failed tests (default: `3`, set to `0` to disable retries)
- `--list` - List all available tests without running them
- `-h, --help` - Show help information
- `-V, --version` - Show version information

### Examples

Run all tests with default settings (auto-detected CPU cores):
```bash
miden-client-integration-tests
```

Run tests sequentially (no parallelism):
```bash
miden-client-integration-tests --jobs 1
```

Run tests with custom parallelism:
```bash
miden-client-integration-tests --jobs 8
```

List all available tests without running them:
```bash
miden-client-integration-tests --list
```

Run only client-related tests:
```bash
miden-client-integration-tests --filter "client"
```

Run tests containing "fpi" in their name:
```bash
miden-client-integration-tests --contains "fpi"
```

Exclude swap-related tests:
```bash
miden-client-integration-tests --exclude "swap"
```

Run tests against devnet:
```bash
miden-client-integration-tests --network devnet
```

Run tests against testnet:
```bash
miden-client-integration-tests --network testnet
```

Run tests against a custom RPC endpoint with timeout:
```bash
miden-client-integration-tests --network http://192.168.1.100:57291 --timeout 30000
```

Complex example: Run non-swap tests in parallel excluding swap tests:
```bash
miden-client-integration-tests --exclude "swap"
```

Show help:
```bash
miden-client-integration-tests --help
```

## Test Categories

The integration tests cover several categories:

- **Client**: Basic client functionality, account management, and note handling
- **Custom Transaction**: Custom transaction types and Merkle store operations
- **FPI**: Foreign Procedure Interface tests
- **Network Transaction**: Network-level transaction processing
- **Onchain**: On-chain account and note operations
- **Swap Transaction**: Asset swap functionality

## Test Case Generation

The integration tests use an automatic code generation system to create both `cargo nextest` compatible tests and a standalone binary. Test functions that start with `test_` are automatically discovered during build time and used to generate:

1. **Individual `#[tokio::test]` wrappers** - These allow the tests to be run using standard `cargo test` or `cargo nextest run` commands
2. **Programmatic test access** - A `Vec<TestCase>` that enables the standalone binary to enumerate and execute tests dynamically with custom parallelism and filtering

The discovery system:
- Scans all `.rs` files in the `src/` directory recursively
- Identifies functions named `test_*` (supporting `pub async fn test_*`, `async fn test_*`, etc.)
- Generates test registry and integration test wrappers automatically

This dual approach allows the same test code to work seamlessly with both nextest (for development) and the standalone binary (for CI/CD and production testing scenarios), ensuring consistent behavior across different execution environments.

## Writing Tests

To add a new integration test:

1. Create a public async function that starts with `test_`
2. The function should take a `ClientConfig` parameter
3. The function should return `Result<()>`
4. Place the function in any `.rs` file under `src/`

Example:
```rust
pub async fn test_my_feature(client_config: ClientConfig) -> Result<()> {
    let (mut client, authenticator) = client_config.into_client().await?;
    // test logic here
}
```

The build system will automatically discover this function and include it in both the test registry and generate tokio test wrappers.

## License
This project is [MIT licensed](../../LICENSE).
