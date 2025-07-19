# Overview

This document describes the current state of the organization of integration tests, along with info on how to run them.

## Running integration tests

There are commands provided in the `Makefile` to make running them easier. To run the current integration test, you should run:

```bash
# This command will run the node
make start-node
```

And on a second terminal do:

```bash
# This will run the integration test
make integration-test-full
```

Note that in order to run this as part of a CI/CD workflow (at least on github), you'll need to use `make start-node-background` instead so the process keeps running on background.

To stop the background node, you can use the `make stop-node` command.

## Running tests with Genesis Configuration

The node builder now supports custom genesis configurations, which allows you to pre-fund accounts and set up initial state without requiring setup transactions in your tests.

To run the node with a custom genesis configuration:

```bash
# Start node with custom genesis configuration
make start-node-with-genesis

# Or in background
make start-node-background-with-genesis
```

This uses the genesis configuration file at `tests/config/genesis_swap_test.toml`, which sets up:
- Two faucet accounts (BTC and ETH mock tokens)
- Two wallet accounts pre-funded with tokens from these faucets

### Example: Swap Tests

The swap integration tests (`tests/src/swap_transactions_tests.rs`) have been refactored to use pre-funded accounts from genesis. This eliminates the need for:
- Creating faucet accounts during the test
- Minting and distributing tokens to test accounts
- Waiting for multiple setup transactions

To run the swap tests with genesis configuration:

1. Start the node with genesis config:
   ```bash
   make start-node-background-with-genesis
   ```

2. Run the swap tests:
   ```bash
   cargo test -p miden-client-tests swap --release
   ```

3. Stop the node when done:
   ```bash
   make stop-node
   ```

## Integration Test Flow

The integration test goes through a series of supported flows such as minting and transferring assets which runs against a running node.

### Setup

Before running the tests though, there is a setup we need to perform to have a node up and running. This is accomplished with the `start-node` command from the `Makefile` and what it does is:

- Delete previously existing data
- Uses the `NodeBuilder` to create a running node in `localhost:57291`.

Killing the node process after running the test is also the user's responsibility. This can be done by using `ctrl + c` in the terminal where the node is running, or by using the `make stop-node` command if it's running in the background.

### Test Run

To run the integration test you just need to run `make integration-test`. It'll run the integration tests as a cargo test using the `integration` feature which is used to separate regular tests from integration tests.

### Ignored Tests

Currently, we have one ignored test because it requires having the genesis data
from the node it is running against which might not always be possible. You can
run it manually by doing:

```bash
cargo nextest run --release --test=integration --features integration --run-ignored ignored-only -- import_genesis_accounts_can_be_used_for_transactions
```

Or run `make integration-test-full` to run all integration tests with
that included. On the other hand, if you want to run integration tests without
that one you can just instead do:

```bash
make integration-test
```

### Running tests against a remote node

You can run the integration tests against a remote node by overwriting the rpc section of the configuration file at `./config/miden-client-rpc.toml`.

## CI integration

There is a step for the CI at `../.github/workflows/ci.yml` used to run the integration tests.
