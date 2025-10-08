# Working with the mocked web client

The mock web client is useful for testing and development purposes, as it simulates interactions with the Miden blockchain without requiring a live network connection. This allows you to experiment with account creation, transaction execution, and note management in a controlled, faster environment.

The mock web client mimics the normal `WebClient` interface to allow for seamless integration with existing code, the only difference is the addition of a `proveBlock` function. The simulated environment will not create blocks automatically, so `proveBlock` needs to be called manually each time a new block should be created.

```typescript
import { MockWebClient, TransactionProver } from "@demox-labs/miden-sdk";

try {
    // Initialize the mock web client
    const mockWebClient = await MockWebClient.createClient();

    // Running a mint transaction (assuming the client was setup previously)
    const pipeline = await mockWebClient.executeTransaction(
      faucetAccount.id(),
      mintTransactionRequest
    );

    await pipeline.proveTransaction(TransactionProver.newLocalProver());
    const mintTransactionUpdate = await pipeline.submitProvenTransaction();
    await mockWebClient.applyTransaction(mintTransactionUpdate);

    await mockWebClient.proveBlock(); // Creates a new block that will include the submitted transaction
    await mockWebClient.syncState();

} catch (error) {
    console.error("An error occurred while using the mock web client:", error.message);
}
```

## Relevant Documentation

For more detailed information about the classes and methods used in these examples, refer to the following API documentation:

- [WebClient](docs/src/web-client/api/classes/WebClient.md) - Main client class for interacting with the Miden network.

For a complete list of available classes and utilities, see the [SDK API Reference](docs/src/web-client/api/README.md).
