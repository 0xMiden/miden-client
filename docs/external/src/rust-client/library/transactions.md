---
title: Transactions
sidebar_position: 7
---

# Retrieving Transaction History

This guide demonstrates how to retrieve and inspect transaction history using the Miden Client Rust library.

## List all transactions

```rust
let transactions = client.get_transactions().await?;

for tx in &transactions {
    println!("TX ID: {:?}", tx.id());
    println!("Account: {:?}", tx.account_id());
    println!("Status: {:?}", tx.transaction_status());
}
```

## Transaction statuses

Transactions follow this lifecycle:

| Status | Description |
|--------|-------------|
| `Pending` | Transaction has been submitted but not yet confirmed |
| `Committed` | Transaction has been included in a block |
| `Discarded` | Transaction was not included and will not be processed |

After executing and submitting a transaction, it starts as `Pending`. Call `sync_state()` to update the status. Once the node includes the transaction in a block, it becomes `Committed`.

## Output notes

Transactions may produce output notes (e.g., a pay-to-id transaction creates a note the recipient can consume):

```rust
let output_notes = client.get_output_notes().await?;

for note in &output_notes {
    println!("Output note: {:?}", note.id());
}
```
