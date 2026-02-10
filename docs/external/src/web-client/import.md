---
title: import
sidebar_position: 3
---

# Importing Data with the Miden SDK

This guide demonstrates how to import accounts, notes, and store data using the Miden SDK.

## Importing Accounts

### Importing by Account ID

Import a public account by its ID (fetches state from the network):

```typescript
import { MidenClient } from "@miden-sdk/miden-sdk";

try {
    const client = await MidenClient.create();

    const account = await client.accounts.import("0x1234...");
    console.log("Imported account:", account.id().toString());
} catch (error) {
    console.error("Failed to import account:", error.message);
}
```

### Importing from an Account File

Import an account that was previously exported:

```typescript
import { MidenClient } from "@miden-sdk/miden-sdk";

try {
    const client = await MidenClient.create();

    // accountFile should be the result of a previous client.accounts.export()
    const account = await client.accounts.import({ file: accountFile });
    console.log("Imported account:", account.id().toString());
} catch (error) {
    console.error("Failed to import account:", error.message);
}
```

### Importing a Public Account from Seed

Import a public account using an initialization seed:

```typescript
import { MidenClient, AuthScheme } from "@miden-sdk/miden-sdk";

try {
    const client = await MidenClient.create();

    const account = await client.accounts.import({
        seed: initSeed,           // Uint8Array
        mutable: true,            // Whether account code can be updated
        auth: AuthScheme.Falcon   // Optional auth scheme
    });
    console.log("Imported account:", account.id().toString());
} catch (error) {
    console.error("Failed to import public account:", error.message);
}
```

## Importing Notes

```typescript
import { MidenClient } from "@miden-sdk/miden-sdk";

try {
    const client = await MidenClient.create();

    // noteFile should be the result of a previous client.notes.export()
    const noteId = await client.notes.import(noteFile);
    console.log("Imported note:", noteId);
} catch (error) {
    console.error("Failed to import note:", error.message);
}
```

### Note File Types

There are three types of note files:

1. **ID Note File** — Contains only the note ID and metadata
2. **Full Note File** — Contains complete note data including content and inclusion proof
3. **Details Note File** — Contains the note ID, metadata, and note details

## Importing Store Data

To import an entire store snapshot (overwrites current store):

```typescript
import { MidenClient } from "@miden-sdk/miden-sdk";

try {
    const client = await MidenClient.create();

    // snapshot should be the result of a previous client.exportStore()
    await client.importStore(snapshot);
    console.log("Store imported successfully");
} catch (error) {
    console.error("Failed to import store:", error.message);
}
```

:::warning
`importStore` is a destructive operation that completely overwrites the current store. Ensure you have a backup if needed.
:::
