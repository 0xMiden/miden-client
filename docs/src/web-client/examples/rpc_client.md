# RPC client

The RPC client is a separate component that allows direct interaction with a Miden node via its RPC interface. This is useful for advanced users who need to query for specific blockchain data.

```typescript
import { Endpoint, RpcClient, NoteId } from "@demox-labs/miden-sdk";

try {
    // Create the RPC client. NOTE: this assumes the node is running on localhost
    const endpoint = new Endpoint("http://localhost:57291");
    const rpcClient = new RpcClient(endpoint);

    // Query the node
    const noteId = NoteId.fromHex(_consumedNoteId);
    const fetchedNotes = await rpcClient.getNotesById([noteId]);

} catch (error) {
    console.error("An error occurred while using the RPC client:", error.message);
}
```

## Relevant Documentation

For more detailed information about the classes and methods used in these examples, refer to the following API documentation:

- [Endpoint](../api/classes/Endpoint.md) - Represents a network endpoint for the RPC client.
- [RpcClient](../api/classes/RpcClient.md) - Client for interacting with a Miden node via RPC.
- [NoteId](../api/classes/NoteId.md) - Represents a unique identifier for a note.

For a complete list of available classes and utilities, see the [SDK API Reference](../api/README.md).

