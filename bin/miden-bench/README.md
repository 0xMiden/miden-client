# miden-bench

Benchmarking tool for the Miden client library. This binary measures performance of transactions to establish baselines and identify optimization opportunities.

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

Deploys a public wallet with configurable storage to the network. This is a prerequisite for running `transaction` benchmarks.

```bash
miden-bench --network localhost deploy --maps 2 --entries-per-map 50
```

The command outputs the account ID and seed, along with a ready-to-copy `transaction` command:

```
Account ID: 0xabcdef1234567890...
Seed: 0123456789abcdef...

Run benchmarks with:
  miden-bench transaction --account-id 0xabcdef1234567890 --seed 0123456789abcdef... --entries-per-map 50
```

**Deployment modes:**

- **Single transaction** (≤200 total entries): all storage entries are included in the initial account.
- **Two-phase** (>200 total entries): deploys an account with sentinel entries, then expands storage via batched transactions. Storage map values are random.

**Limits:**

*gRPC Message Size:* The default node gRPC limit is 4MB. Accounts above ~200 total entries use two-phase deployment (deploy + expand via batched transactions).

*Memory Requirements:* Transaction proving is memory-intensive:
- Small (10-50 total entries): Works on any machine
- Medium (100-200 total entries): Works on most machines
- Large (500+ total entries): Requires ~32GB RAM and increased gRPC limits

### `transaction`

Benchmarks transaction operations that read storage from an account (requires a running Miden node):

- **execute** - Measures transaction execution time (without proving) reading all storage entries
- **prove** - Measures transaction proving time and proof size
- **full** - Measures full transaction (execute + prove + submit)

The benchmark executes transactions that read all storage map entries from the specified account. The account must be a public account deployed to the network via `deploy`.

The number of storage maps is auto-detected from the account. The `--entries-per-map` flag controls how many entries per map are read:

- **Small accounts** (single-tx deployment, ≤200 entries): `--entries-per-map` can be omitted — entries are read directly from the imported account.
- **Large accounts** (two-phase deployment, >200 entries): `--entries-per-map` is required. The node's import RPC only returns the initial component entries, not entries added by expansion transactions. The `deploy` command prints the correct value in its suggested command.

```bash
# First deploy an account with storage
miden-bench --network localhost deploy --maps 2 --entries-per-map 50
# Copy the transaction command from the output

# Then benchmark transactions against it
miden-bench --network localhost transaction --account-id 0x... --seed <hex> --entries-per-map 50
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

### Iterations (`-i, --iterations`)

Number of measured iterations per benchmark. Default: 5.

```bash
miden-bench --iterations 20 transaction --account-id 0x... --seed <hex>
```

### Deploy Options

- `-m, --maps <N>` - Number of storage maps in the account (default: 1)
- `-e, --entries-per-map <N>` - Number of key/value entries per storage map (default: 10)

```bash
# Deploy account with 3 storage maps, each with 100 entries (300 total)
miden-bench deploy --maps 3 --entries-per-map 100
```

### Transaction Options

- `-a, --account-id <ID>` - Public account ID to benchmark against (required, hex format)
- `-s, --seed <HEX>` - Account seed for signing (hex-encoded 32 bytes, output by `deploy`). When omitted, only execution is benchmarked (no proving or submission).
- `-e, --entries-per-map <N>` - Entries per map (required for two-phase deployed accounts, optional for small accounts)

## Examples

### Full workflow: deploy and benchmark

```bash
# Deploy an account with 2 maps of 100 entries each
miden-bench --network localhost deploy --maps 2 --entries-per-map 100

# Copy the transaction command from the output and run it
miden-bench --network localhost transaction --account-id 0x... --seed <hex> --entries-per-map 100
```

## Metrics

Each benchmark reports:

- **Mean** - Average duration across all iterations
- **Min/Max** - Range of observed values
- **Output Size** - Size in bytes (for proving benchmarks)
