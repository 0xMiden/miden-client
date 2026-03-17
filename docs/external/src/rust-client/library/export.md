---
title: Export
sidebar_position: 5
---

# Exporting Data

This guide demonstrates how to export accounts and notes from the Miden Client.

## Exporting notes

Export a note by its ID:

```rust
let note_data = client.export_note(note_id).await?;
```

Notes can be exported in different formats depending on how much data to include:

| Format | Description |
|--------|-------------|
| ID only | Contains only the note ID (works for public notes that can be fetched from the network) |
| With details | Contains the note ID, metadata, and creation block number |
| Full | Contains the complete note with its inclusion proof |

Exported notes can be shared with other users and [imported](./import.md) into their clients.

## Exporting accounts

Retrieve the full account state for export:

```rust
let account = client.get_account(account_id).await?;
```

The returned `Account` object includes the full account state, code, and vault. It can be serialized and shared with another client via [import](./import.md).

:::tip
For public accounts, the recipient can simply [import by ID](./import.md#import-by-account-id) instead of needing an exported file.
:::
