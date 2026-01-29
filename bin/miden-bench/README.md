# miden-bench

Benchmarking tool for the Miden client library. This binary measures performance of transactions, account export/import, and network synchronization to establish baselines and identify optimization opportunities.

## Installation

```bash
cargo install miden-client-bench
```

### From source

```bash
# Using make
make install-bench

# Or directly with cargo
cargo install --path bin/miden-bench --locked
```

After installation, the `miden-bench` binary will be available in your PATH.

## Commands

### `deploy`

Deploys a public wallet with configurable storage to the network. This is useful for creating test accounts that can then be used with the `sync --account-id` benchmark.

```bash
miden-bench --network localhost deploy --size medium
```

The command outputs the account ID which can be used with `sync --account-id`:

```bash
# First deploy an account
miden-bench --network localhost deploy --size medium
# Account ID: 0xabcdef1234567890...

# Then benchmark importing it
miden-bench --network localhost sync --account-id 0xabcdef1234567890
```

**Note:** Transaction proving is memory-intensive. Recommended limits:
- `small` (10 entries): Works on any machine
- `medium` (100 entries): Works on most machines
- `large` (1,000 entries): Requires ~32GB RAM
- `very-large` (50,000 entries): Requires ~128GB+ RAM

For benchmarking large account imports, deploy on a high-memory machine first, then run sync benchmarks on any machine.

### `sync`

Benchmarks network synchronization operations (requires a running Miden node):

- **no_account_tracking** - Measures sync time with no accounts being tracked
- **import_public_account** - Measures time to import a public account from the network (requires `--account-id`)

```bash
# Basic sync benchmark
miden-bench --network localhost sync

# With public account import benchmark
miden-bench --network localhost sync --account-id 0xabcdef1234567890
```

### `export`

Benchmarks account serialization and deserialization:

- **serialize_account** - Measures time to serialize an `AccountFile` to bytes
- **deserialize_account** - Measures time to deserialize bytes back to an `AccountFile`
- **roundtrip_account** - Measures combined serialize + deserialize time

```bash
miden-bench export --size medium
```

### `transaction`

Benchmarks transaction execution with large storage accounts (requires a running Miden node):

- **execute** - Measures transaction execution time (without proving) with target account having configured storage entries
- **prove** - Measures transaction proving time and proof size
- **full** - Measures full transaction (execute + prove + submit)

```bash
miden-bench --network localhost transaction --size medium
```

## Global Options

These options apply to all commands and must be specified before the subcommand.

### Network Environment (`-n, --network`)

Specifies the network environment for benchmarks that require a Miden node. Default: `localhost`.

| Value | Endpoint |
|-------|----------|
| `localhost` (or `local`) | `http://localhost:57291` |
| `devnet` (or `dev`) | `https://rpc.devnet.miden.io` |
| `testnet` (or `test`) | `https://rpc.testnet.miden.io` |
| Custom URL | Any valid RPC endpoint |

```bash
# Use local development node
miden-bench --network localhost sync

# Use Miden devnet
miden-bench --network devnet sync

# Use custom endpoint
miden-bench --network https://my-node.example.com:8080 sync
```

You can also set the network via the `MIDEN_NETWORK` environment variable:

```bash
export MIDEN_NETWORK=devnet
miden-bench sync
```

### Iterations (`-i, --iterations`)

Number of measured iterations per benchmark. Default: 5.

```bash
miden-bench --iterations 20 export --size medium
```

## Subcommand Options

Controls the set data used for the benchmark. These options are specific to each subcommand and must be specified after the subcommand.

```bash
# Run with large accounts
miden-bench export --size large

# Run with very-large accounts (stress test)
miden-bench export --size very-large
```

## Examples

### Quick validation (small accounts, few iterations)

```bash
miden-bench --iterations 3 export --size small
```

### Stress test with very-large accounts

```bash
miden-bench --iterations 5 export --size very-large
```

## Metrics

Each benchmark reports:

- **Mean** - Average duration across all iterations
- **Min/Max** - Range of observed values
- **Output Size** - Size in bytes (for serialization benchmarks)
