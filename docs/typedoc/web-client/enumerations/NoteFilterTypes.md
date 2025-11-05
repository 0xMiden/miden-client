[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / NoteFilterTypes

# Enumeration: NoteFilterTypes

Enumerates the different note filter variants supported by the client.

## Enumeration Members

### All

> **All**: `0`

Return all notes.

***

### Committed

> **Committed**: `2`

Only include notes that are committed.

***

### Consumed

> **Consumed**: `1`

Only include notes that were consumed.

***

### Expected

> **Expected**: `3`

Only include notes that are expected.

***

### List

> **List**: `5`

Filter to a specific list of note IDs.

***

### Nullifiers

> **Nullifiers**: `7`

Filter by note nullifiers (currently unused placeholder).

***

### Processing

> **Processing**: `4`

Only include notes currently being processed.

***

### Unique

> **Unique**: `6`

Filter to a single unique note ID.

***

### Unverified

> **Unverified**: `8`

Only include notes that are unverified.
