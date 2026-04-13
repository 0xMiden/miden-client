---
title: Export
sidebar_position: 5
---

# Exporting Data

This guide demonstrates how to export accounts and notes from the Miden Client.

## Exporting notes

Retrieve an output note and convert it to a `NoteFile` for export:

```rust
use miden_client::note::NoteExportType;

let note_record = client.get_output_note(note_id).await?
    .expect("note exists");

let note_file = note_record.into_note_file(&NoteExportType::Full)?;
```

Notes can be exported in different formats depending on how much data to include:

| `NoteExportType` | Description |
|--------|-------------|
| `Id` | Contains only the note ID (works for public notes that can be fetched from the network) |
| `Details` | Contains the note ID, metadata, and creation block number |
| `Full` | Contains the complete note with its inclusion proof |

The resulting `NoteFile` can be serialized and shared with other users for [import](./import.md) into their clients.

## Exporting accounts

Retrieve the full account state for export:

```rust
let account = client.get_account(account_id).await?
    .expect("account exists");
```

The returned `Account` object includes the full account state, code, and vault. It can be serialized and shared with another client via [import](./import.md).

:::tip
For public accounts, the recipient can simply [import by ID](./import.md#import-by-account-id) instead of needing an exported file.
:::
