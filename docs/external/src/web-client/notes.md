---
title: notes
sidebar_position: 5
---

# Working with Notes in the Miden SDK

This guide demonstrates how to work with notes in the Miden SDK. Notes are the primary way to transfer assets and data between accounts in the Miden network.

## Retrieving Input Notes

Input notes are notes that can be consumed (spent) in transactions. You can retrieve them individually or in bulk:

```typescript
import { NoteFilter, NoteFilterTypes, WebClient } from "@miden-sdk/miden-sdk";

try {
    // Initialize the web client
    const webClient = await WebClient.createClient();

    // Get a single input note by ID
    const note = await webClient.getInputNote(noteId);
    if (note) {
        console.log("Note ID:", note.id().toString());
    }

    // Get all input notes
    const filter = new NoteFilter(NoteFilterTypes.All);
    const notes = await webClient.getInputNotes(filter);
    notes.forEach(note => {
        console.log("Note ID:", note.id().toString());
    });
} catch (error) {
    console.error("Failed to retrieve notes:", error.message);
}
```

## Retrieving Output Notes

Output notes are notes that have been created by transactions. You can retrieve them individually or in bulk:

```typescript
import { NoteFilter, NoteFilterTypes, WebClient } from "@miden-sdk/miden-sdk";

try {
    // Initialize the web client
    const webClient = await WebClient.createClient();

    // Get a single output note by ID
    const noteId = await webClient.getOutputNote(noteId);
    console.log("Note ID:", noteId);

    // Get all output notes
    const filter = new NoteFilter(NoteFilterTypes.All);
    const noteIds = await webClient.getOutputNotes(filter);
    noteIds.forEach(id => {
        console.log("Note ID:", id);
    });
} catch (error) {
    console.error("Failed to retrieve notes:", error.message);
}
```

## Working with Consumable Notes

Consumable notes are notes that can be spent by a specific account. You can retrieve them with or without filtering by account:

```typescript
import { AccountId, WebClient } from "@miden-sdk/miden-sdk";

try {
    // Initialize the web client
    const webClient = await WebClient.createClient();

    // Get consumable notes for a specific account
    const accountId = AccountId.fromHex(accountIdHex);
    const records = await webClient.getConsumableNotes(accountId);
    
    records.forEach(record => {
        console.log("Note ID:", record.inputNoteRecord().id().toString());
        record.noteConsumability().forEach(consumability => {
            console.log("Account ID:", consumability.accountId().toString());
            console.log("Consumable after block:", consumability.consumableAfterBlock());
        });
    });

    // Get all consumable notes
    const allRecords = await webClient.getConsumableNotes();
    // ... process records as above
} catch (error) {
    console.error("Failed to retrieve consumable notes:", error.message);
}
```

## Compiling Note Scripts

You can compile custom note scripts for advanced use cases:

```typescript
import { CodeBuilder, CodeBuilderMode } from "@miden-sdk/miden-sdk";

try {
    // Instance a script builder
    const webClient = await WebClient.createClient();

    const script = `
        # Your custom note script here
        # This can include custom validation logic, asset transfers, etc.
    `;

    // Script builder can be instanced in either debug or normal mode using one of:
    // - CodeBuilderMode.Debug
    // - CodeBuilderMode.Normal
    
    const noteScript = new CodeBuilder(CodeBuilderMode.Debug);
    // Use the compiled script in your transaction
} catch (error) {
    console.error("Failed to compile note script:", error.message);
}
```

## Relevant Documentation

For more detailed information about note functionality, refer to the following API documentation:

- [WebClient](https://github.com/0xMiden/miden-client/docs/typedoc/web-client/classes/WebClient.md) - Main client class for note operations
- [NoteFilter](https://github.com/0xMiden/miden-client/docs/typedoc/web-client/classes/NoteFilter.md) - Class for filtering notes
- [NoteFilterTypes](https://github.com/0xMiden/miden-client/docs/typedoc/web-client/enumerations/NoteFilterTypes.md) - Enumeration for note filter types
- [AccountId](https://github.com/0xMiden/miden-client/docs/typedoc/web-client/classes/AccountId.md) - Class for working with account IDs
- [InputNoteRecord](https://github.com/0xMiden/miden-client/docs/typedoc/web-client/classes/InputNoteRecord.md) - Class representing input notes
- [ConsumableNoteRecord](https://github.com/0xMiden/miden-client/docs/typedoc/web-client/classes/ConsumableNoteRecord.md) - Class representing consumable notes
- [NoteScript](https://github.com/0xMiden/miden-client/docs/typedoc/web-client/classes/NoteScript.md) - Class for working with note scripts

For a complete list of available classes and utilities, see the [SDK API Reference](https://github.com/0xMiden/miden-client/docs/typedoc/web-client/README.md).

### Available Note Filter Types

The `NoteFilter` supports several filter types to help you retrieve specific sets of notes:

- `NoteFilterTypes.All`: Retrieves all notes
- `NoteFilterTypes.Consumed`: Retrieves only consumed (spent) notes
- `NoteFilterTypes.Committed`: Retrieves only committed notes
- `NoteFilterTypes.Expected`: Retrieves only expected notes
- `NoteFilterTypes.Processing`: Retrieves notes that are currently being processed
- `NoteFilterTypes.List`: Retrieves specific notes by their IDs (requires passing note IDs)
- `NoteFilterTypes.Unique`: Retrieves a single note by its ID (requires passing exactly one note ID)
- `NoteFilterTypes.Nullifiers`: Retrieves notes by their nullifiers
- `NoteFilterTypes.Unverified`: Retrieves unverified notes

Example of using a specific filter type:

```typescript
// Get only consumed notes
const consumedFilter = new NoteFilter(NoteFilterTypes.Consumed);
const consumedNotes = await webClient.getInputNotes(consumedFilter);

// Get specific notes by their IDs
const noteIds = [noteId1, noteId2];
const listFilter = new NoteFilter(NoteFilterTypes.List, noteIds);
const specificNotes = await webClient.getInputNotes(listFilter);
``` 
