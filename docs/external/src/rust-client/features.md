---
title: Features
sidebar_position: 3
---

The `miden-client` library provides a comprehensive Rust API for interacting with the Miden rollup.

### Transaction execution

Build and execute transactions programmatically using `TransactionRequestBuilder`. The library supports standard transaction types (pay-to-id, swap, mint) through convenience builders, and custom transactions via arbitrary note scripts and arguments.

```rust
let tx_request = TransactionRequestBuilder::new()
    .build_pay_to_id(payment_description, None, NoteType::Private, client.rng())?;

let result = client.new_transaction(sender_id, tx_request).await?;
client.submit_transaction(result).await?;
```

### Proof generation

Generate client-side zero-knowledge proofs that validate transactions without revealing private data. The library supports:

- **Local proving** — Generate proofs on the client machine using `LocalTransactionProver`
- **Remote proving** — Delegate proof generation to a remote prover via `RemoteTransactionProver`
- **Prover fallback** — Configure a remote prover as default with automatic fallback to local proving on failure

The prover is a pluggable trait, allowing custom implementations.

### Network interactivity

Communicate with the Miden network through a gRPC-based RPC client:

- **State sync** — `client.sync_state().await` fetches the latest blockchain state, updating accounts, notes, and transactions
- **Note transport** — Exchange private notes via the note transport network using `SendNote` and `FetchNotes` gRPC methods
- **Configurable endpoints** — Connect to testnet, devnet, localhost, or custom network endpoints

### Account management

Create and manage accounts using `AccountBuilder`:

- **Account types** — `RegularAccountImmutableCode`, `RegularAccountUpdatableCode`, fungible/non-fungible faucets
- **Storage modes** — `Private` (state tracked locally, only commitments on-chain) or `Public` (state stored on-chain)
- **Key management** — Trait-based keystore for signing and authentication (filesystem keystore provided)
- **Components** — Attach `BasicWallet`, `AuthRpoFalcon512`, and custom components to accounts

### Extensibility

The library is designed around trait-based dependency injection:

- **Store** — Pluggable persistence (SQLite implementation provided)
- **RPC client** — Pluggable network communication
- **Prover** — Pluggable proof generation
- **Keystore** — Pluggable key management

This makes every component replaceable for testing, custom backends, or alternative environments.
