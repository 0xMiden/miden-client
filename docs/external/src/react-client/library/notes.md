---
title: Notes
sidebar_position: 4
---

# Notes

## useNotes(options?)

List and filter notes with consumability information.

```tsx
import { useNotes, formatNoteSummary } from "@miden-sdk/react";

function NoteList({ accountId }: { accountId: string }) {
  const {
    notes,                    // All input notes
    consumableNotes,          // Notes ready to consume
    noteSummaries,            // Enriched with sender + assets
    consumableNoteSummaries,  // Consumable notes with summaries
    isLoading,
    error,
    refetch,
  } = useNotes({ accountId });

  return (
    <div>
      {consumableNoteSummaries.map((summary) => (
        <div key={summary.id}>{formatNoteSummary(summary)}</div>
      ))}
    </div>
  );
}
```

### Filter options

| Property | Type | Description |
|----------|------|-------------|
| `status` | `string` | `"all"` \| `"committed"` \| `"consumed"` \| `"expected"` \| `"processing"` |
| `accountId` | `string` | Filter consumable notes for a specific account |
| `sender` | `string` | Filter by sender account |
| `excludeIds` | `Set<string>` | Exclude specific note IDs |

Smart refetch: the hook only updates the Zustand store if note IDs actually changed, preventing unnecessary re-renders.

## useNoteStream(options?)

Temporal note tracking with built-in filtering. Designed for real-time UIs (e.g., payment notifications, trading interfaces) where you need to track when notes arrive and handle them progressively.

```tsx
import { useNoteStream } from "@miden-sdk/react";

function NotificationFeed({ accountId }: { accountId: string }) {
  const {
    notes,            // StreamedNote[] matching all filters
    latest,           // Most recent note
    markHandled,      // Exclude noteId from future renders
    markAllHandled,   // Mark all current notes as handled
    snapshot,         // Get { ids: Set, timestamp } for phase transitions
    isLoading,
  } = useNoteStream({
    status: "committed",
    sender: faucetId,
    since: Date.now() - 60_000,  // Last minute only
    amountFilter: (amount) => amount > 0n,
  });

  return (
    <div>
      {notes.map((note) => (
        <div key={note.id}>
          {note.amount.toString()} from {note.sender}
          <button onClick={() => markHandled(note.id)}>Dismiss</button>
        </div>
      ))}
    </div>
  );
}
```

### StreamedNote

| Property | Type | Description |
|----------|------|-------------|
| `id` | `string` | Note ID |
| `sender` | `string` | Sender address (bech32) |
| `amount` | `bigint` | Primary fungible asset amount |
| `assets` | `NoteAsset[]` | All assets in the note |
| `record` | `InputNoteRecord` | Full note record |
| `firstSeenAt` | `number` | When the SDK first observed this note (ms) |
| `attachment` | `bigint[] \| null` | Pre-decoded note attachment |

### Options

| Property | Type | Description |
|----------|------|-------------|
| `status` | `string` | Filter by note status |
| `sender` | `string` | Filter by sender account |
| `since` | `number` | Only notes seen after this timestamp (ms) |
| `excludeIds` | `Set<string>` | Exclude specific note IDs |
| `amountFilter` | `(amount: bigint) => boolean` | Custom amount filter |
