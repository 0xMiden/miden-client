---
title: note-transport
sidebar_position: 10
---

# Note Transport with the Miden SDK

This guide demonstrates how to use the note transport features in the Miden SDK. Note transport allows you to send and receive private notes using the Miden Note Transport network.

## Sending Private Notes

```typescript
import { MidenClient } from "@miden-sdk/miden-sdk";

try {
    const client = await MidenClient.create();

    // Send a private note to a recipient address
    await client.notes.sendPrivate({
        noteId: "0xnote...",           // ID of the note to send
        to: "mtst1recipient..."        // Recipient bech32 address
    });

    console.log("Private note sent successfully");
} catch (error) {
    console.error("Failed to send private note:", error.message);
}
```

## Fetching Private Notes

```typescript
import { MidenClient } from "@miden-sdk/miden-sdk";

try {
    const client = await MidenClient.create();

    // Fetch private notes using pagination (default)
    await client.notes.fetch();

    // Or fetch all private notes at once
    // (reserve for special cases like initial setup)
    await client.notes.fetch({ mode: "all" });

    // List the fetched notes
    const notes = await client.notes.list();
    console.log(`Fetched ${notes.length} notes`);
} catch (error) {
    console.error("Failed to fetch private notes:", error.message);
}
```
