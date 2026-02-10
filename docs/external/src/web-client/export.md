---
title: export
sidebar_position: 4
---

# Exporting Data with the Miden SDK

This guide demonstrates how to export accounts, notes, and store data using the Miden SDK.

## Exporting Notes

Export a note with different levels of detail:

```typescript
import { MidenClient } from "@miden-sdk/miden-sdk";

try {
    const client = await MidenClient.create();

    // Export with just the note ID
    const idExport = await client.notes.export("0xnote...", { format: "id" });

    // Export with full details including inclusion proof
    const fullExport = await client.notes.export("0xnote...", { format: "full" });

    // Export with partial details (default if format is not specified)
    const partialExport = await client.notes.export("0xnote...", { format: "partial" });
} catch (error) {
    console.error("Failed to export note:", error.message);
}
```

Export formats:

- `"id"` — Exports only the note ID (only works for public notes)
- `"full"` — Exports the complete note with its inclusion proof (requires the note to have an inclusion proof)
- `"partial"` — Exports note details including metadata and the creation block number

## Exporting Accounts

```typescript
import { MidenClient } from "@miden-sdk/miden-sdk";

try {
    const client = await MidenClient.create();

    const accountFile = await client.accounts.export("0x1234...");
    console.log("Account exported:", accountFile);
} catch (error) {
    console.error("Failed to export account:", error.message);
}
```

Account files include the full account state, code, seed (if new), and tracked secret keys.

## Exporting the Store

Export the entire client store for backup or migration:

```typescript
import { MidenClient } from "@miden-sdk/miden-sdk";

try {
    const client = await MidenClient.create();

    const snapshot = await client.exportStore();
    console.log("Store exported:", snapshot);

    // The snapshot has the shape { version: 1, data: ... }
    // You can save this and later restore with client.importStore(snapshot)
} catch (error) {
    console.error("Failed to export store:", error.message);
}
```
