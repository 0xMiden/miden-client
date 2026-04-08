---
title: tags
sidebar_position: 7
---

# Working with Note Tags in the Miden SDK

Note tags are used to specify how notes should be executed and who can consume them. They also serve as a fuzzy filter mechanism for retrieving note updates during sync operations.

## Basic Tag Operations

```typescript
import { MidenClient } from "@miden-sdk/miden-sdk";

try {
    const client = await MidenClient.create();

    // Add a tag (accepts number)
    await client.tags.add(12345);

    // Remove a tag
    await client.tags.remove(12345);

    // List all tags (returns number[])
    const tags = await client.tags.list();
    console.log("Current tags:", tags);
} catch (error) {
    console.error("Failed to manage tags:", error.message);
}
```

## Managing Multiple Tags

```typescript
import { MidenClient } from "@miden-sdk/miden-sdk";

try {
    const client = await MidenClient.create();

    // Add multiple tags
    const tagsToAdd = [123, 456, 789];
    for (const tag of tagsToAdd) {
        await client.tags.add(tag);
    }

    // List them
    const allTags = await client.tags.list();
    console.log("All tags:", allTags); // [123, 456, 789]

    // Remove some
    await client.tags.remove(123);
    await client.tags.remove(456);
} catch (error) {
    console.error("Failed to manage tags:", error.message);
}
```

## Tag Sources and Sync Behavior

Tags can come from different sources:

1. **Account Tags**: Automatically added for accounts being tracked by the client.
2. **Note Tags**: Automatically added for expected notes being tracked.
3. **User Tags**: Manually added by the user via `client.tags.add()`.

During sync operations, the client uses these tags to retrieve note-related information for notes with matching tags.

## Important Notes

- Tags are `number` values (valid `u32` numbers)
- Tags for managed accounts are handled automatically by the client
- User-added tags can be removed, but system-generated tags cannot
- Use `NoteTag` helpers from WASM to compute tag values from faucet IDs
