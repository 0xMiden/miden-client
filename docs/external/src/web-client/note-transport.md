---
title: note-transport
sidebar_position: 10
---

# Note Transport with the Miden SDK

This guide demonstrates how to use the note transport features in the Miden SDK. Note transport allows you to send and receive private notes using the Miden Note Transport network.

## Initializing the Client with Note Transport

To use note transport features, you need to initialize the client with a note transport endpoint URL:

```typescript
import { WebClient } from "@miden-sdk/miden-sdk";

try {
    // Initialize the web client with note transport endpoint
    const webClient = await WebClient.createClient({
        rpcUrl: undefined,                      // Miden node endpoint (optional, defaults to testnet)
        noteTransportUrl: "https://transport.miden.io", // Miden Note Transport node (optional, defaults to none)
        seed: undefined                         // seed (optional)
    });

    console.log("Client initialized with note transport");
} catch (error) {
    console.error("Failed to initialize client:", error.message);
}
```

## Sending Private Notes

To send a private note, use `sendPrivateNote()`:

```typescript
import { WebClient, Note, Address } from "@miden-sdk/miden-sdk";

try {
    // Initialize the web client with note transport endpoint
    const webClient = await WebClient.createClient({
        rpcUrl: undefined,
        noteTransportUrl: "https://transport.miden.io",
        seed: undefined
    });

    const note = /* note to be sent here */;
    const address = /* recipient's address here */;

    // Send the private note
    await webClient.sendPrivateNote(note, address);

    console.log("Private note sent successfully");
} catch (error) {
    console.error("Failed to send private note:", error.message);
}
```

## Fetching Private Notes

To fetch private notes from the note transport layer, use `fetchPrivateNotes()`.

```typescript
import { WebClient, NoteFilter, NoteFilterTypes } from "@miden-sdk/miden-sdk";

try {
    // Initialize the web client with note transport endpoint
    const webClient = await WebClient.createClient({
        rpcUrl: undefined,
        noteTransportUrl: "https://transport.miden.io",
        seed: undefined
    });

    // Fetch private notes using pagination
    await webClient.fetchPrivateNotes();

    // Alternatively, fetch all private notes without pagination.
    // Reserve this for special cases like initial setup or troubleshooting.
    // await webClient.fetchAllPrivateNotes();

    // Retrieve the fetched notes
    const filter = new NoteFilter(NoteFilterTypes.All);
    const notes = await webClient.getInputNotes(filter);

    console.log(`Fetched ${notes.length} private notes`);
} catch (error) {
    console.error("Failed to fetch private notes:", error.message);
}
```

## Relevant Documentation

For more detailed information about note transport functionality, refer to the following API documentation:

- [WebClient](https://github.com/0xMiden/miden-client/docs/typedoc/web-client/classes/WebClient.md) - Main client class for note transport operations
- [Note](https://github.com/0xMiden/miden-client/docs/typedoc/web-client/classes/Note.md) - Class for working with notes
- [Address](https://github.com/0xMiden/miden-client/docs/typedoc/web-client/classes/Address.md) - Class for working with addresses
- [NoteFilter](https://github.com/0xMiden/miden-client/docs/typedoc/web-client/classes/NoteFilter.md) - Class for filtering notes
- [NoteFilterTypes](https://github.com/0xMiden/miden-client/docs/typedoc/web-client/enumerations/NoteFilterTypes.md) - Enumeration for note filter types

For a complete list of available classes and utilities, see the [SDK API Reference](https://github.com/0xMiden/miden-client/docs/typedoc/web-client/README.md).
