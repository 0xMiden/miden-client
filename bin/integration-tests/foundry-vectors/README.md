# Foundry Test Vector Generator

Solidity-based test vector generator for the agglayer bridge integration tests. Uses Foundry to simulate L1 `bridgeAsset()` transactions and produce JSON files containing valid Merkle proofs, leaf data, and exit roots that the Rust integration tests consume.

## Prerequisites

- [Foundry](https://book.getfoundry.sh/getting-started/installation) (`forge`)

## Setup

Install dependencies:

```bash
forge install
```

## Usage

### Generate claim asset test vectors (default destination)

```bash
forge test -vv --match-contract ClaimAssetTestVectorsLocalTx
```

This writes `test-vectors/claim_asset_vectors_local_tx.json`.

### Generate claim asset test vectors with a custom destination address

The Rust integration test invokes this via:

```bash
DESTINATION_ADDRESS=0x... forge test -vv --match-contract ClaimAssetTestVectorsLocalTx
```

> **Note:** Environment variable support requires `ffi = true` in `foundry.toml` or passing the address via forge script arguments.

## Output Format

The generated JSON contains:

| Field | Description |
|-------|-------------|
| `leaf_type` | Leaf type (0 = token transfer) |
| `origin_network` | Origin network ID |
| `origin_token_address` | ERC-20 token address on origin chain |
| `destination_network` | Destination network ID (Miden) |
| `destination_address` | Destination address (embeds Miden AccountId) |
| `amount` | Bridge amount in wei |
| `metadata` / `metadata_hash` | ABI-encoded token metadata and its keccak256 hash |
| `leaf_value` | Keccak256 hash of the leaf data |
| `deposit_count` | Number of deposits in the tree |
| `smt_proof_local_exit_root` | 32-element SMT proof for the local exit root |
| `smt_proof_rollup_exit_root` | 32-element SMT proof for the rollup exit root |
| `global_index` | Global index encoding mainnet flag and leaf index |
| `mainnet_exit_root` | Mainnet exit root (= local exit root for L1 deposits) |
| `rollup_exit_root` | Rollup exit root (simulated) |
| `global_exit_root` | `keccak256(mainnetExitRoot \|\| rollupExitRoot)` |

## Project Structure

```
foundry-vectors/
├── foundry.toml                          # Foundry configuration
├── src/
│   └── DepositContractTestHelpers.sol    # SMT proof generation helpers
├── test/
│   └── ClaimAssetTestVectorsLocalTx.t.sol # Test vector generator
├── test-vectors/                          # Generated JSON output (gitignored)
└── lib/                                   # Dependencies (gitignored)
    ├── forge-std/
    ├── zkevm-contracts/
    └── openzeppelin-contracts-upgradeable/
```
