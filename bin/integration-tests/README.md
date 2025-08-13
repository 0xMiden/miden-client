# Miden Client Integration Tests

This directory contains integration tests for the Miden client library. These tests verify the functionality of the client against a running Miden node.

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
- `-h, --help` - Show help information
- `-V, --version` - Show version information

### Examples

Run all tests with default settings:
```bash
miden-client-integration-tests
```

Run tests against a custom RPC endpoint:
```bash
miden-client-integration-tests --rpc-endpoint http://192.168.1.100:57291
```

Run tests with custom timeout:
```bash
miden-client-integration-tests --timeout 30000
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
