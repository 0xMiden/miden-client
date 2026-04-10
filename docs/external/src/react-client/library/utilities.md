---
title: Utilities
sidebar_position: 7
---

# Utilities

## Amount formatting

```tsx
import { formatAssetAmount, parseAssetAmount } from "@miden-sdk/react";

// bigint + decimals → display string
formatAssetAmount(1000000n, 8);    // "0.01"
formatAssetAmount(100000000n, 8);  // "1"

// display string + decimals → bigint
parseAssetAmount("0.01", 8);      // 1000000n
parseAssetAmount("1.5", 8);       // 150000000n
```

## Note summaries

```tsx
import { getNoteSummary, formatNoteSummary } from "@miden-sdk/react";

// Extract summary from an InputNoteRecord
const summary = getNoteSummary(note);
// { id, assets: NoteAsset[], sender?: string }

// Format for display
formatNoteSummary(summary);  // "1.5 TOKEN" or "1.5 TOKEN from miden1qy35..."
```

## Note attachments

```tsx
import { readNoteAttachment, createNoteAttachment } from "@miden-sdk/react";

// Read pre-decoded attachment from a note
const attachment = readNoteAttachment(note);
// { values: bigint[], kind: "word" | "array" } | null

// Create attachment for sending (≤4 values → word, >4 → array)
const data = createNoteAttachment([100n, 200n, 300n]);

// Use in useSend:
await send({ from, to, assetId, amount: 50n, attachment: [100n, 200n] });
```

## Account ID utilities

```tsx
import { normalizeAccountId, accountIdsEqual, toBech32AccountId } from "@miden-sdk/react";

// Normalize hex or bech32 to consistent format
normalizeAccountId("0x1234...");           // normalized bech32

// Compare account IDs regardless of format
accountIdsEqual("0x1234...", "miden1qy35...");  // true

// Convert to bech32
toBech32AccountId("0x1234...");            // "miden1qy35..."
```

## Error handling

The SDK wraps WASM errors into typed `MidenError` objects with actionable error codes.

```tsx
import { MidenError, wrapWasmError } from "@miden-sdk/react";

try {
  await send({ from, to, assetId, amount: 50n });
} catch (err) {
  if (err instanceof MidenError) {
    switch (err.code) {
      case "SEND_BUSY":
        // Transaction already in progress
        break;
      case "WASM_SYNC_REQUIRED":
        // Client needs to sync first
        break;
      case "WASM_NOT_INITIALIZED":
        // Client not ready yet
        break;
    }
  }
}
```

### Error codes

| Code | Meaning |
|------|---------|
| `WASM_CLASS_MISMATCH` | WASM object type mismatch |
| `WASM_POINTER_CONSUMED` | WASM pointer already freed |
| `WASM_NOT_INITIALIZED` | Client not initialized |
| `WASM_SYNC_REQUIRED` | Client needs to sync |
| `SEND_BUSY` | A send operation is already in progress |
| `OPERATION_BUSY` | Another mutation is in progress |
| `UNKNOWN` | Unrecognized error |

## Storage management

```tsx
import { clearMidenStorage, migrateStorage } from "@miden-sdk/react";

// Clear all Miden IndexedDB data
await clearMidenStorage();

// Migrate storage between versions
await migrateStorage({ fromVersion: 1, toVersion: 2 });
```
