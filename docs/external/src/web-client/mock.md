---
title: Working with the mocked web client
draft: true
sidebar_position: 12
---

# Working with the mocked web client

The mock web client is useful for testing and development purposes, as it simulates interactions with the Miden blockchain without requiring a live network connection. This allows you to experiment with account creation, transaction execution, and note management in a controlled, faster environment.

The mock web client mimics the normal `WebClient` interface to allow for seamless integration with existing code. The only difference is the addition of a `proveBlock` function. The simulated environment will not create blocks automatically, so `proveBlock` needs to be called manually each time a new block should be created.

The mock client also includes a simulated note transport layer, allowing you to test note transport features such as `sendPrivateNote()` and `fetchPrivateNotes()` without requiring a live note transport node.

```typescript
import { MockWebClient } from "@miden-sdk/miden-sdk";

try {
  // Initialize the mock web client
  const mockWebClient = await MockWebClient.createClient({});

  const mintTransactionId = await mockWebClient.submitNewTransaction(
    faucetAccount.id(),
    mintTransactionRequest
  );

  console.log("Mint transaction submitted:", mintTransactionId.toString());

  // Advance the mock chain and refresh local state
  await mockWebClient.proveBlock();
  await mockWebClient.syncState();

  const consumableNotes = await mockWebClient.getConsumableNotes(
    userAccount.id()
  );
  const noteIdToConsume = consumableNotes[0]
    .inputNoteRecord()
    .id()
    .toString();
  const inputNoteRecord = await client.getInputNote(noteIdToConsume);
  if (!inputNoteRecord) {
    throw new Error(`Note with ID ${noteIdToConsume} not found`);
  }
  const noteToConsume = inputNoteRecord.toNote();
  const consumeRequest = mockWebClient.newConsumeTransactionRequest([
    noteToConsume,
  ]);

  const consumeTransactionId = await mockWebClient.submitNewTransaction(
    userAccount.id(),
    consumeRequest
  );

  await mockWebClient.proveBlock();
  await mockWebClient.syncState();
} catch (error) {
  console.error(
    "An error occurred while using the mock web client:",
    error.message
  );
}
```

## Working with Note Transport

The mock client includes a built-in note transport layer.

```typescript
import { MockWebClient, NoteFilter, NoteFilterTypes } from "@miden-sdk/miden-sdk";

try {
  // Initialize the mock web client
  const mockWebClient = await MockWebClient.createClient({});

  // Send a private note (example with placeholders)
  const note = /* create your note */;
  const recipientAddress = /* create recipient address */;

  await mockWebClient.sendPrivateNote(note, recipientAddress);

  // Fetch private notes
  await mockWebClient.fetchPrivateNotes();

  // Retrieve the fetched notes
  const filter = new NoteFilter(NoteFilterTypes.All);
  const notes = await mockWebClient.getInputNotes(filter);

  console.log(`Fetched ${notes.length} private notes`);
} catch (error) {
  console.error("Error:", error.message);
}
```

## Relevant Documentation

For more detailed information about the classes and methods used in these examples, refer to the following API documentation:

- [WebClient](../api/classes/WebClient) - Main client class for interacting with the Miden network.

For a complete list of available classes and utilities, see the [SDK API Reference](../api/index).
