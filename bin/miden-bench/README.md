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

The command outputs the account ID, seed, and a ready-to-copy `expand` command:

```
Account ID: 0xabcdef1234567890...
Seed: 0123456789abcdef...

Expand storage with:
  miden-bench expand --account-id 0xabcdef1234567890 --seed 0123456789abcdef... --map-idx 0 --offset 0 --count 100
```

### `expand`

Fills entries into a specific storage map of a deployed account. The account must have been deployed with `deploy` first, which installs the expansion procedures needed to write entries.

```bash
miden-bench expand --account-id 0x... --seed <hex> --map-idx 0 --offset 0 --count 200
```

Entries are batched into transactions of up to 280 entries each. Keys and values are generated deterministically from the map index and entry offset, so repeated runs with the same parameters produce the same data.

To fill multiple maps, run `expand` once per map:

```bash
miden-bench expand --account-id 0x... --seed <hex> --map-idx 0 --offset 0 --count 100
miden-bench expand --account-id 0x... --seed <hex> --map-idx 1 --offset 0 --count 100
```

### `transaction`

Benchmarks transaction operations that read storage from an account (requires a running Miden node):

- **execute** - Measures transaction execution time (without proving) reading all storage entries
- **prove** - Measures transaction proving time and proof size
- **full** - Measures full transaction (execute + prove + submit)

The benchmark executes transactions that read all storage map entries from the specified account. Accounts deployed to the network via `deploy` can test `execute`, `prove` and `full`, while other types of accounts may only measure `execute` times.

The number of storage maps is auto-detected from the account.

```bash
miden-bench --network localhost transaction --account-id 0x... --seed <hex>
```

## Workflow

The typical workflow is: **deploy** -> **expand** -> **transaction**.

```bash
# 1. Deploy an account with 2 empty storage maps
miden-bench --network localhost deploy --maps 2

# 2. Fill each map with entries (copy account-id and seed from deploy output)
miden-bench expand --account-id 0x... --seed <hex> --map-idx 0 --offset 0 --count 100
miden-bench expand --account-id 0x... --seed <hex> --map-idx 1 --offset 0 --count 100

# 3. Benchmark transactions against the account
miden-bench transaction --account-id 0x... --seed <hex>
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

Path to a directory for persistent store data. When provided, `deploy` and `expand` save the SQLite store and keystore in this directory (instead of a temporary one), and `transaction` reuses it — skipping the account import from the node on each iteration.

```bash
# Deploy with a persistent store
miden-bench --store ./bench-data deploy --maps 2

# Expand using the same store
miden-bench --store ./bench-data expand --account-id 0x... --seed <hex> --map-idx 0 --offset 0 --count 100

# Reuse the store for benchmarks (no re-import needed)
miden-bench --store ./bench-data transaction --account-id 0x... --seed <hex>
```

When omitted, temporary directories are used and cleaned up automatically (the default behavior).

### Command Options

#### Deploy

- `-m, --maps <N>` - Number of empty storage maps in the account (1-100, default: 1)

```bash
# Deploy account with 3 empty storage maps
miden-bench deploy --maps 3
```

#### Expand

- `-a, --account-id <ID>` - Public account ID to expand (required, hex format)
- `-s, --seed <HEX>` - Account seed for signing (required, hex-encoded 32 bytes, output by `deploy`)
- `-m, --map-idx <N>` - Storage map index to fill (0-based, must be less than the deploy `--maps` count)
- `-o, --offset <N>` - Starting entry offset (0-based)
- `-c, --count <N>` - Number of entries to add starting from offset

#### Transaction

- `-a, --account-id <ID>` - Public account ID to benchmark against (required, hex format)
- `-s, --seed <HEX>` - Account seed for signing (hex-encoded 32 bytes, output by `deploy`). When omitted, only execution is benchmarked (no proving or submission).
- `-r, --reads <N>` - Maximum storage reads per transaction. When total entries exceed this limit, reads are split across multiple transactions per benchmark iteration. Each iteration's time is the sum across all transactions. When omitted, all entries are read in a single transaction.
- `-i, --iterations <N>` - Number of benchmark iterations (default: 5)

## Metrics

Each benchmark reports a table with columns:

- **Mean** - Average duration across all iterations
- **Min** - Fastest iteration
- **Max** - Slowest iteration

Proving benchmarks also display the proof output size alongside the benchmark name.
