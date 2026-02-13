# miden-bench

Benchmarking tool for the Miden client library. This binary measures performance of transactions to establish baselines and identify optimization opportunities.

## Installation

```bash
# Using make
make install-bench

# Or directly with cargo
cargo install --path bin/miden-bench --locked
```

After installation, the `miden-bench` binary will be available in your PATH.

## Commands

### `deploy`

Deploys a public wallet with empty storage maps to the network. This is the first step in preparing an account for benchmarking. Storage maps are created empty; use `expand` to fill them with entries.

```bash
miden-bench --network localhost deploy --maps 2
```

The command outputs the account ID and a ready-to-copy `expand` command:

```
Account ID: 0xabcdef1234567890...

Expand storage with:
  miden-bench expand --account-id 0xabcdef1234567890 --map-idx 0 --offset 0 --count 100
```

### `expand`

Fills entries into a specific storage map of a deployed account. The account must have been deployed with `deploy` first, which installs the expansion procedures needed to write entries.

```bash
miden-bench expand --account-id 0x... --map-idx 0 --offset 0 --count 200
```

Entries are batched into transactions of up to 280 entries each. Keys and values are generated deterministically from the map index and entry offset, so repeated runs with the same parameters produce the same data.

To fill multiple maps, run `expand` once per map:

```bash
miden-bench expand --account-id 0x... --map-idx 0 --offset 0 --count 100
miden-bench expand --account-id 0x... --map-idx 1 --offset 0 --count 100
```

### `transaction`

Benchmarks transaction operations that read storage from an account (requires a running Miden node):

- **execute** - Measures transaction execution time (without proving) reading all storage entries
- **prove** - Measures transaction proving time and proof size
- **full** - Measures full transaction (execute + prove + submit)

The benchmark executes transactions that read all storage map entries from the specified account. Accounts deployed via `deploy` automatically have the signing key persisted, enabling all three benchmarks (`execute`, `prove`, and `full`). If the signing key is not found (e.g., account was imported from the network without deploying), only `execute` is benchmarked.

The number of storage maps is auto-detected from the account.

```bash
miden-bench --network localhost transaction --account-id 0x...
```

## Workflow

The typical workflow is: **deploy** -> **expand** -> **transaction**.

All commands share a persistent store directory (default: `./miden-bench-store`). The `deploy` command saves the account and signing key there, and `expand` and `transaction` reuse them automatically.

```bash
# 1. Deploy an account with 2 empty storage maps
miden-bench --network localhost deploy --maps 2

# 2. Fill each map with entries (copy account-id from deploy output)
miden-bench expand --account-id 0x... --map-idx 0 --offset 0 --count 100
miden-bench expand --account-id 0x... --map-idx 1 --offset 0 --count 100

# 3. Benchmark transactions against the account
miden-bench transaction --account-id 0x...
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
miden-bench --network localhost deploy

# Use Miden devnet
miden-bench --network devnet deploy
```

You can also set the network via the `MIDEN_NETWORK` environment variable:

```bash
export MIDEN_NETWORK=devnet
miden-bench deploy
```

### Persistent Store (`--store`)

Path to a directory for persistent store data. Default: `./miden-bench-store`.

All commands use this directory for the `SQLite` database and filesystem keystore. The `deploy` command creates and populates it; `expand` and `transaction` reuse it.

```bash
# Use a custom store directory
miden-bench --store ./my-bench-data deploy --maps 2
miden-bench --store ./my-bench-data expand --account-id 0x... --map-idx 0 --offset 0 --count 100
miden-bench --store ./my-bench-data transaction --account-id 0x...
```

### Command Options

#### Deploy

- `-m, --maps <N>` - Number of empty storage maps in the account (1-100, default: 1)

```bash
# Deploy account with 3 empty storage maps
miden-bench deploy --maps 3
```

#### Expand

- `-a, --account-id <ID>` - Public account ID to expand (required, hex format)
- `-m, --map-idx <N>` - Storage map index to fill (0-based, must be less than the deploy `--maps` count)
- `-o, --offset <N>` - Starting entry offset (0-based)
- `-c, --count <N>` - Number of entries to add starting from offset

#### Transaction

- `-a, --account-id <ID>` - Public account ID to benchmark against (required, hex format)
- `-r, --reads <N>` - Maximum storage reads per transaction. When total entries exceed this limit, reads are split across multiple transactions per benchmark iteration. Each iteration's time is the sum across all transactions. When omitted, all entries are read in a single transaction.
- `-i, --iterations <N>` - Number of benchmark iterations (default: 5)

## Metrics

Each benchmark reports a table with columns:

- **Mean** - Average duration across all iterations
- **Min** - Fastest iteration
- **Max** - Slowest iteration

Proving benchmarks also display the proof output size alongside the benchmark name.
