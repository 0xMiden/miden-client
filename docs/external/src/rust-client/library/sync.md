---
title: Sync
sidebar_position: 11
---

# Synchronizing State

This guide demonstrates how to synchronize local state with the Miden network using the Miden Client Rust library.

## Basic sync

```rust
let sync_summary = client.sync_state().await?;

println!("Synced to block: {}", sync_summary.block_num);
println!("New public notes: {}", sync_summary.new_public_notes);
println!("Committed notes: {}", sync_summary.committed_notes);
println!("Consumed notes: {}", sync_summary.consumed_notes);
println!("Updated accounts: {}", sync_summary.updated_accounts);
println!("Committed transactions: {}", sync_summary.committed_transactions);
```

## When to sync

Sync after any operation that depends on network state:

- **After submitting a transaction** — to confirm it was committed
- **Before consuming notes** — to verify notes are committed on-chain
- **After importing a note** — to transition it from `Expected` to `Committed`
- **Periodically** — to discover new public notes or track account updates

## What sync updates

The sync process queries the Miden node and updates local state:

| Updated | Description |
|---------|-------------|
| Block headers | Latest chain state |
| Account state | Public account updates from the node |
| Note status | Transitions: `Expected` → `Committed`, `Processing` → `Consumed` |
| Transaction status | Transitions: `Pending` → `Committed` (or `Discarded`) |
| Note tags | Discovers new notes matching tracked tags |

## Get current sync height

Check the last synced block number without performing a sync:

```rust
let height = client.get_sync_height().await?;
println!("Last synced block: {}", height);
```
