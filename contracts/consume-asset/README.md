# Consume Asset — Transaction Script

A Miden transaction script that allows a solver to consume spread assets directly into the executing account's vault.

## How It Works

1. The caller passes a **commitment word** (RPO hash) as the transaction-script argument.
2. The script loads the corresponding advice-map value, which is verified against the commitment.
3. Each 4-felt word in the payload is interpreted as an `Asset` and received into the account's vault via `account.receive_asset()`.

## Usage

This script is used by solvers during cross-swap settlement. When two swap notes produce a spread (the difference between offered and requested amounts), the solver captures that spread by:

- Computing the commitment over the spread assets
- Pushing the asset data into the advice map
- Executing this script to receive the spread into the solver's account

## Building

```sh
cargo miden build --release
```

The compiled `.masp` file is referenced by the `miden-swapp` crate via `include_bytes!`.

## Dependencies

Depends on `basic-wallet` for the wallet interface (`Account::receive_asset`).
