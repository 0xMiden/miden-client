---
title: Import
sidebar_position: 4
---

# Importing Data

This guide demonstrates how to import accounts and notes into the Miden Client.

## Importing accounts

### Import by account ID

Import a public account from the network by its ID:

```rust
let account_id = AccountId::from_hex("0x1234...")?;
client.import_account_by_id(account_id).await?;
```

This fetches the account's current state from the chain and stores it locally for tracking.

### Import from an Account object

Add an existing `Account` object to the client's store:

```rust
client.add_account(&account, false).await?;
```

The second parameter controls whether to overwrite an existing account with the same ID.

:::warning[Public accounts only]
Import-by-ID only works for **public accounts** — those created with `AccountStorageMode::Public`. Private account state is not available on-chain, so it cannot be fetched by ID. To transfer a private account between clients, export and import the `Account` object directly.
:::

## Importing notes

### Import from note files

Import notes using `NoteFile` variants:

```rust
use miden_client::note::NoteFile;

let note_files = vec![
    NoteFile::NoteId(note_id),  // Fetch full note from network
];

let imported_ids = client.import_notes(&note_files).await?;
println!("Imported {} notes", imported_ids.len());
```

### Note file types

| Variant | Description |
|---------|-------------|
| `NoteFile::NoteId` | Contains only the note ID; full note is fetched from the network |
| `NoteFile::NoteDetails` | Contains note details, metadata, and creation block number |
| `NoteFile::NoteWithProof` | Contains the complete note with its inclusion proof |

### Import from a `.mno` file

Import a note exported as a `.mno` file (e.g., from the faucet):

```rust
use std::path::Path;

client.import_note_from_file(Path::new("path/to/note.mno")).await?;
```

After importing, [sync](./sync.md) the client to verify the note's inclusion on-chain. The note's status transitions from `Expected` to `Committed`.
