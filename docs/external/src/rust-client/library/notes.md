---
title: Notes
sidebar_position: 9
---

# Working with Notes

Notes are the primary way to transfer assets and data between accounts in the Miden network. This guide demonstrates how to work with notes using the Miden Client Rust library.

For importing and exporting notes, see [Import](./import.md) and [Export](./export.md). For exchanging private notes between users, see [Note Transport](./note-transport.md).

## List input notes

```rust
// List all input notes
let notes = client.get_input_notes(NoteFilter::All).await?;

for note in &notes {
    println!("Note ID: {:?}", note.id());
    println!("Status: {:?}", note.status());
}
```

## Filter notes by status

```rust
use miden_client::note::NoteFilter;

let committed = client.get_input_notes(NoteFilter::Committed).await?;
let consumed = client.get_input_notes(NoteFilter::Consumed).await?;
let expected = client.get_input_notes(NoteFilter::Expected).await?;
let processing = client.get_input_notes(NoteFilter::Processing).await?;
```

### Note statuses

| Status | Description |
|--------|-------------|
| `Expected` | Note has been imported or created but not yet confirmed on-chain |
| `Committed` | Note has been confirmed on-chain (after sync) |
| `Processing` | Note consumption proof has been submitted, awaiting confirmation |
| `Consumed` | Note has been consumed (after sync confirms the consumption) |

## Get consumable notes

Find notes that a specific account can consume:

```rust
let consumable = client.get_consumable_notes(Some(account_id)).await?;

for record in &consumable {
    println!("Note: {:?}", record.note.id());
}
```

Pass `None` to get consumable notes across all tracked accounts.

## List output notes

Output notes are notes created by your transactions (e.g., the P2ID note created when sending tokens):

```rust
let output_notes = client.get_output_notes(NoteFilter::All).await?;

for note in &output_notes {
    println!("Output note: {:?}", note.id());
}
