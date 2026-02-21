---
title: notes
sidebar_position: 5
---

# Working with Notes in the Miden SDK

This guide demonstrates how to work with notes in the Miden SDK. Notes are the primary way to transfer assets and data between accounts in the Miden network.

## Listing Notes

```typescript
import { MidenClient } from "@miden-sdk/miden-sdk";

try {
    const client = await MidenClient.create();

    // List all input notes (received)
    const allNotes = await client.notes.list();

    // Filter by status
    const committed = await client.notes.list({ status: "committed" });
    const consumed = await client.notes.list({ status: "consumed" });
    const expected = await client.notes.list({ status: "expected" });
    const processing = await client.notes.list({ status: "processing" });
    const unverified = await client.notes.list({ status: "unverified" });

    // Filter by specific IDs
    const specific = await client.notes.list({ ids: [noteId1, noteId2] });

    for (const note of allNotes) {
        console.log("Note ID:", note.id().toString());
    }
} catch (error) {
    console.error("Failed to retrieve notes:", error.message);
}
```

## Retrieving a Single Note

```typescript
import { MidenClient } from "@miden-sdk/miden-sdk";

try {
    const client = await MidenClient.create();

    // Get a single input note by ID (returns null if not found)
    const note = await client.notes.get("0xnote...");
    if (note) {
        console.log("Note ID:", note.id().toString());
    }
} catch (error) {
    console.error("Failed to retrieve note:", error.message);
}
```

## Listing Sent Notes (Output Notes)

```typescript
import { MidenClient } from "@miden-sdk/miden-sdk";

try {
    const client = await MidenClient.create();

    // List all sent (output) notes
    const sentNotes = await client.notes.listSent();

    // Filter sent notes by status
    const committedSent = await client.notes.listSent({ status: "committed" });

    for (const note of sentNotes) {
        console.log("Sent Note ID:", note.id().toString());
    }
} catch (error) {
    console.error("Failed to retrieve sent notes:", error.message);
}
```

## Listing Consumable Notes

Consumable notes are notes that can be spent by a specific account:

```typescript
import { MidenClient } from "@miden-sdk/miden-sdk";

try {
    const client = await MidenClient.create();
    const wallet = await client.accounts.create();

    // Get consumable notes for a specific account
    const records = await client.notes.listAvailable({ account: wallet });

    for (const record of records) {
        console.log("Note ID:", record.inputNoteRecord().id().toString());
        for (const consumability of record.noteConsumability()) {
            console.log("Account ID:", consumability.accountId().toString());
        }
    }
} catch (error) {
    console.error("Failed to retrieve consumable notes:", error.message);
}
```

## Importing and Exporting Notes

```typescript
import { MidenClient } from "@miden-sdk/miden-sdk";

try {
    const client = await MidenClient.create();

    // Import a note from a note file
    const noteId = await client.notes.import(noteFile);
    console.log("Imported note:", noteId);

    // Export a note - different formats available
    const idExport = await client.notes.export("0xnote...", { format: "id" });
    const fullExport = await client.notes.export("0xnote...", { format: "full" });
    const detailsExport = await client.notes.export("0xnote...", { format: "details" });
} catch (error) {
    console.error("Failed to import/export note:", error.message);
}
```

Export formats:
- `"id"` — Exports only the note ID (only works for public notes)
- `"full"` — Exports the complete note with its inclusion proof
- `"details"` — Exports note details including metadata and creation block

## Available Note Filter Statuses

When listing notes, you can filter by these statuses:

- `"committed"` — Notes committed to the blockchain
- `"consumed"` — Notes that have been spent
- `"expected"` — Notes expected to arrive
- `"processing"` — Notes currently being processed
- `"unverified"` — Unverified notes

Or filter by specific IDs using `{ ids: [noteId1, noteId2] }`.
