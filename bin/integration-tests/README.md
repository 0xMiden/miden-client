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

- `-r, --rpc-endpoint <URL>` - The URL of the RPC endpoint to use (default: `http://localhost:57291`)
- `-t, --timeout <MILLISECONDS>` - Timeout for RPC requests in milliseconds (default: `10000`)
- `-j, --jobs <NUMBER>` - Number of tests to run in parallel (default: auto-detected CPU cores, set to `1` for sequential execution)
- `-f, --filter <REGEX>` - Filter tests by name using regex patterns
- `--contains <STRING>` - Only run tests whose names contain this substring
- `--exclude <REGEX>` - Exclude tests whose names match this regex pattern
- `--list` - List all available tests without running them
- `-v, --verbose` - Show verbose output including individual test timings and worker information
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

Run tests with verbose output showing worker information:
```bash
miden-client-integration-tests --verbose
```

Run tests against a custom RPC endpoint with timeout:
```bash
miden-client-integration-tests --rpc-endpoint http://192.168.1.100:57291 --timeout 30000
```

Complex example: Run non-swap tests in parallel with verbose output:
```bash
miden-client-integration-tests --exclude "swap" --verbose
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
