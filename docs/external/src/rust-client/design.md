---
title: Design
sidebar_position: 4
---

The Miden client has the following architectural components:

- [Store](#store)
- [RPC client](#rpc-client)
- [Transaction executor](#transaction-executor)
- [Keystore](#keystore)
- [Note screener](#note-screener)
- [Note transport](#note-transport)

:::tip

- The RPC client and the store are Rust traits.
- This allow developers and users to easily customize their implementations.

:::

## Store

The store is central to the client's design.

It manages the persistence of the following entities:

- Accounts; including their state history and related information such as vault assets and account code.
- Transactions and their scripts.
- Notes.
- Note tags.
- Block headers and chain information that the client needs to execute transactions and consume notes.

Because Miden allows off-chain executing and proving, the client needs to know about the state of the blockchain at the moment of execution. To avoid state bloat, however, the client does not need to see the whole blockchain history, just the chain history intervals that are relevant to the user.

The store can track any number of accounts, and any number of notes that those accounts might have created or may want to consume.

## RPC client

The RPC client communicates with the node through a defined set of gRPC methods. The provided client works both in `std` and `wasm` environments.

The available gRPC methods are documented in the [Node gRPC Reference](https://docs.miden.xyz/miden-node/rpc).

## Transaction executor

The transaction executor uses the [Miden VM](https://0xmiden.github.io/miden-docs/imported/miden-vm/src/intro/main.html) to execute transactions. All transactions run within the [transaction kernel](https://0xmiden.github.io/miden-docs/imported/miden-base/src/transaction.html).

When executing, the executor needs access to relevant blockchain history. The executor uses a `DataStore` interface for accessing this data. This means that there may be some coupling between the executor and the store.

## Keystore

The keystore is responsible for storing and managing the private keys of the accounts tracked by the client.

These private keys are used by the executor to sign and authenticate transactions. Implementations for both rust and web keystores are provided.

## Note Screener

The note screener determines which notes are relevant to the client by checking whether any tracked account can consume them. It does this by performing dry runs of consumption transactions using the `NoteConsumptionChecker` from the `miden-tx` crate internally.

### Core functionality

The `NoteScreener` is constructed with a reference to the client's store and RPC client:

```rust
let screener = NoteScreener::new(store, rpc_api);
```

It provides three main methods:

- **`can_consume(note)`**: Checks whether any tracked account can consume a single note. Returns a list of `(AccountId, NoteConsumptionStatus)` pairs for each account that could consume it.
- **`can_consume_batch(notes)`**: Checks a batch of notes against all tracked accounts. Returns a map from `NoteId` to the list of accounts that can consume each note.
- **`check_notes_consumability(account_id, notes)`**: Checks a set of notes against a specific account by attempting to execute them together. Returns a `NoteConsumptionInfo` that splits notes into those that succeeded and those that failed.

### Transaction arguments

The screener supports an optional builder method for providing custom transaction arguments:

```rust
let screener = NoteScreener::new(store, rpc_api)
    .with_transaction_args(tx_args);
```

If not set, the screener uses default `TransactionArgs` with an empty advice map.

### Sync integration

The `NoteScreener` implements the `OnNoteReceived` trait, which serves as a callback during state sync. When the client syncs with the network and receives note commitment updates, this callback determines what to do with each note:

- If the note is already tracked (as an input or output note), it is committed.
- If the note is public and matches a tracked note tag, it is inserted.
- If the note is public and untracked, the screener checks its consumability — if any tracked account can consume it, the note is inserted; otherwise it is discarded.
- If the note is private and untracked, it is discarded since its contents cannot be inspected.

The callback returns a `NoteUpdateAction` enum (`Commit`, `Insert`, or `Discard`) that instructs the sync component on how to handle the note.

Custom implementations of `OnNoteReceived` can be provided to the `StateSync` component to override this default behavior.

### Security note

During consumability checks, the screener deliberately does **not** attach a real authenticator to the transaction executor. This prevents external signers (e.g. wallet extensions) from being invoked during sync, which would produce unwanted confirmation popups. The `NoteConsumptionChecker` handles the missing authenticator gracefully by returning `ConsumableWithAuthorization` instead of calling `get_signature()`.

## State Sync component

The state sync component encapsulates the logic for dealing with synchronization of the client state with the network. It repeatedly queries the node with sync state requests until the chain tip is reached. On every requests it updates the provided tracked elements (accounts, notes, transactions, etc.) and returns an updated state at the end which can be used to update the store (this component does not modify the store directly).

The component also exposes a specific customizable callback which can be used to react to new note arrivals.

## Note transport

Access to the note transport network to exchange private notes is also provided.
The provided client uses gRPC methods to communicate with the note transport network, working both in `std` and `wasm` environments.

Targeting privacy, notes are primarily exchanged using their tags as identifiers. By default, when notes are created the tag is derived from the recipient account ID, however the tag can also be random.

The system is also prepared for end-to-end encryption (to be implemented).

gRPC methods include:

- `SendNote`: Sends a note to the note transport network. The recipient address is employed to encrypt the outgoing note (to be implemented).
- `FetchNotes`: Fetch notes from the network by note tag. A pagination mechanism using a monotonic-increasing cursor is also employed. The cursor is created by the network and used by the client to reduce the number of fetched notes (to avoid downloading already fetched notes).
