---
title: Examples
sidebar_position: 6
---

# Examples

Code examples for common Rust library operations, organized by topic.

## Tutorials

For guided, end-to-end walkthroughs, see the [Getting started](./get-started/index.md) tutorials:

- [Create account and use faucet](./get-started/create-account-use-faucet.md)
- [Public peer-to-peer transfer](./get-started/p2p-public.md)
- [Private peer-to-peer transfer](./get-started/p2p-private.md)

## Library reference examples

Each library reference page contains code examples for its topic:

### Account operations

- [Creating wallets, contracts, and faucets](./library/new-accounts.md)
- [Retrieving accounts and checking balances](./library/accounts.md)
- [Importing accounts](./library/import.md)
- [Exporting accounts](./library/export.md)

### Transaction operations

- [Minting, sending, consuming, and swapping](./library/new-transactions.md) — includes remote prover, custom scripts, and swaps
- [Retrieving transaction history](./library/transactions.md)

### Note operations

- [Listing, filtering, and consuming notes](./library/notes.md)
- [Sending and fetching private notes](./library/note-transport.md)

### Other

- [Compiling MASM components and transaction scripts](./library/compile.md)
- [Synchronizing state](./library/sync.md)
- [Managing note tags](./library/tags.md)

## Prover fallback pattern

When using a remote prover, network issues or server errors may cause proving to fail. A common pattern is to configure the client with a remote prover by default and fall back to local proving when remote proving fails.

```rust
use std::sync::Arc;
use miden_client::{
    ClientError,
    RemoteTransactionProver,
    builder::ClientBuilder,
    transaction::{LocalTransactionProver, ProvingOptions},
};

// Create provers
let remote_prover = Arc::new(RemoteTransactionProver::new("https://prover.example.com"));
let local_prover = Arc::new(LocalTransactionProver::new(ProvingOptions::default()));

// Build client with remote prover as default
let mut client = ClientBuilder::new()
    .prover(remote_prover.clone())
    .store(store)
    .rpc(rpc)
    .authenticator(authenticator)
    .build()
    .await?;

// Build transaction request
let tx_request = /* ... build your transaction request ... */;

// Submit with fallback: try remote prover first, fall back to local on proving error
let tx_id = match client.submit_new_transaction(account_id, tx_request.clone()).await {
    Ok(id) => id,
    Err(ClientError::TransactionProvingError(_)) => {
        println!("Remote proving failed, falling back to local prover...");
        client
            .submit_new_transaction_with_prover(account_id, tx_request, local_prover.clone())
            .await?
    }
    Err(e) => return Err(e.into()),
};

println!("Transaction submitted: {}", tx_id);
```
