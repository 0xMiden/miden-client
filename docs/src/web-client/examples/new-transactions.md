# Creating Transactions with the Miden SDK

This guide demonstrates how to create and submit different types of transactions using the Miden SDK. We'll cover minting, sending, consuming, and custom transactions.

## Basic Transaction Flow

All transactions follow a similar pattern:
1. Create a transaction request
2. Execute the transaction pipeline to perform local validation and execution
3. Prove the transaction (locally or by using a remote prover)
4. Submit the proven transaction to the network and apply the resulting update

Here's a basic example of how to execute and submit a mint transaction to mint tokens from a faucet:

```typescript
import { NoteType, TransactionProver, WebClient } from "@demox-labs/miden-sdk";

try {
    // Initialize the web client
    const webClient = await WebClient.createClient();

    const transactionRequest = webClient.newMintTransactionRequest(
        targetAccountId, // AccountId: The account that will receive the minted tokens
        faucetId,// AccountId: The faucet account that will mint the tokens
        NoteType.Private, // NoteType: The type of note to create (Private or Public)
        1000 // number: The amount of tokens to mint
    );

    // 2. Execute the transaction pipeline (performs request validation and execution)
    const pipeline = await webClient.executeTransactionPipeline(
        accountId,
        transactionRequest
    );

    // Inspect execution results before proving
    const executedTx = pipeline.executedTransaction();
    console.log("Created notes:", executedTx.outputNotes());
    console.log("Consumed notes:", executedTx.inputNotes());
    console.log("Account delta:", executedTx.accountDelta());

    // 3. Generate a proof and submit the transaction
    await pipeline.proveTransaction(TransactionProver.newLocalProver());
    const transactionUpdate = await pipeline.submitProvenTransaction();
    await webClient.applyTransaction(transactionUpdate);

    console.log("Block number:", transactionUpdate.blockNum());
    console.log(
        "Submitted transaction:",
        transactionUpdate.executedTransaction().id().toHex()
    );
} catch (error) {
    console.error("Transaction failed:", error.message);
}
```

### Using a Remote Prover

For better performance, you can offload the work of proving the transaction to a remote prover. This is especially useful for complex transactions:

```typescript
import { NoteType, TransactionProver, WebClient } from "@demox-labs/miden-sdk";

try {
    const webClient = await WebClient.createClient();

    const remoteProver = TransactionProver.newRemoteProver("https://prover.example.com");

    const transactionRequest = webClient.newMintTransactionRequest(
        targetAccountId,
        faucetId,
        NoteType.Private,
        1000
    );

    const pipeline = await webClient.executeTransactionPipeline(
        accountId,
        transactionRequest
    );

    await pipeline.proveTransaction(remoteProver);
    const transactionUpdate = await pipeline.submitProvenTransaction();
    await webClient.applyTransaction(transactionUpdate);

    console.log("Block number:", transactionUpdate.blockNum());
    console.log(
        "Submitted transaction:",
        transactionUpdate.executedTransaction().id().toHex()
    );
} catch (error) {
    console.error("Transaction failed:", error.message);
}
```

> **Note**: Using a remote prover can significantly improve performance for complex transactions by offloading the computationally intensive proving work to a dedicated server. This is particularly useful when dealing with large transactions or when running in resource-constrained environments.

## Sending Transactions

To send tokens between accounts:

```typescript
import { NoteType, TransactionProver, WebClient } from "@demox-labs/miden-sdk";

try {
    // Initialize the web client
    const webClient = await WebClient.createClient();

    const transactionRequest = webClient.newSendTransactionRequest(
        senderAccountId,  // Account sending tokens
        targetAccountId,  // Account receiving tokens
        faucetId,        // Faucet account ID
        NoteType.Private, // Note type
        100,             // Amount to send
        100,             // Optional recall height
        90               // Optional timelock height
    );

    const pipeline = await webClient.executeTransactionPipeline(
        senderAccountId,
        transactionRequest
    );

    await pipeline.proveTransaction(TransactionProver.newLocalProver());
    const transactionUpdate = await pipeline.submitProvenTransaction();
    await webClient.applyTransaction(transactionUpdate);

    console.log("Block number:", transactionUpdate.blockNum());
    console.log("Created notes:", transactionUpdate.executedTransaction().outputNotes());
    console.log("Consumed notes:", transactionUpdate.executedTransaction().inputNotes());
    console.log("Account delta:", transactionUpdate.executedTransaction().accountDelta());
} catch (error) {
    console.error("Send transaction failed:", error.message);
}
```

## Consuming Notes

To consume (spend) notes:

```typescript
import { TransactionProver, WebClient } from "@demox-labs/miden-sdk";

try {
    // Initialize the web client
    const webClient = await WebClient.createClient();

    const transactionRequest = webClient.newConsumeTransactionRequest(
        [noteId1, noteId2]  // Array of note IDs to consume
    );

    const pipeline = await webClient.executeTransactionPipeline(
        accountId,
        transactionRequest
    );

    await pipeline.proveTransaction(TransactionProver.newLocalProver());
    const transactionUpdate = await pipeline.submitProvenTransaction();
    await webClient.applyTransaction(transactionUpdate);

    console.log("Block number:", transactionUpdate.blockNum());
    console.log("Created notes:", transactionUpdate.executedTransaction().outputNotes());
    console.log("Consumed notes:", transactionUpdate.executedTransaction().inputNotes());
    console.log("Account delta:", transactionUpdate.executedTransaction().accountDelta());
} catch (error) {
    console.error("Consume transaction failed:", error.message);
}
```

## Custom Transactions

For advanced use cases, you can create custom transactions by defining your own note scripts and transaction parameters. This allows for:

- Custom note validation logic
- Complex asset transfers
- Custom authentication schemes
- Integration with smart contracts

> **Note**: For a complete example of a custom transaction implementation, including input notes, output notes, and custom scripts, see the integration tests in [`new_transactions.test.ts`](https://github.com/0xMiden/miden-client/blob/main/crates/web-client/test/new_transactions.test.ts).

Here's a simplified example of creating a custom transaction:

```typescript
import { 
    Felt, 
    FeltArray,
    FungibleAsset,
    NotesArray,
    NoteAssets,
    NoteExecutionHint,
    NoteExecutionMode,
    NoteMetadata, 
    NoteTag,
    NoteType, 
    OutputNotesArray,
    TransactionProver,
    TransactionRequestBuilder,
    TransactionScript,
    WebClient
} from "@demox-labs/miden-sdk";

try {
    // Initialize the web client
    const webClient = await WebClient.createClient();

    // Create note assets
    const noteAssets = new NoteAssets([
        new FungibleAsset(faucetId, BigInt(10))
    ]);

    // Create note metadata
    const noteMetadata = new NoteMetadata(
        faucetId,
        NoteType.Private,
        NoteTag.fromAccountId(targetAccountId, NoteExecutionMode.newLocal()),
        NoteExecutionHint.none()
    );

    // Create note arguments
    const noteArgs = [new Felt(BigInt(9)), new Felt(BigInt(12))];
    const feltArray = new FeltArray();
    noteArgs.forEach(felt => feltArray.append(felt));

    // Create custom note script
    const noteScript = `
        # Your custom note script here
        # This can include custom validation logic, asset transfers, etc.
    `;

    // Create transaction script
    const transactionScript = new TransactionScript(noteScript);

    // Create output notes array
    const outputNotes = new OutputNotesArray();
    // Add your output notes here

    // Create expected notes array
    const expectedNotes = new NotesArray();
    // Add your expected notes here

    // Build the transaction request
    const transactionRequest = new TransactionRequestBuilder()
        .withCustomScript(transactionScript)
        .withOwnOutputNotes(outputNotes)
        .withExpectedOutputNotes(expectedNotes)
        .build();

    // Create and submit the transaction
    const pipeline = await webClient.executeTransactionPipeline(
        accountId,
        transactionRequest
    );

    await pipeline.proveTransaction(TransactionProver.newLocalProver());
    const transactionUpdate = await pipeline.submitProvenTransaction();
    await webClient.applyTransaction(transactionUpdate);
    
    // Access transaction details
    console.log("Block number:", transactionUpdate.blockNum());
    console.log("Created notes:", transactionUpdate.executedTransaction().outputNotes());
    console.log("Consumed notes:", transactionUpdate.executedTransaction().inputNotes());
    console.log("Account delta:", transactionUpdate.executedTransaction().accountDelta());
} catch (error) {
    console.error("Custom transaction failed:", error.message);
}
```

> **Note**: Custom transactions require a good understanding of the Miden VM and its instruction set. They are powerful but should be used with caution as they can affect the security and correctness of your application.

## Relevant Documentation

For more detailed information about transaction functionality, refer to the following API documentation:

- [WebClient](docs/src/web-client/api/classes/WebClient.md) - Main client class for transaction operations
- [TransactionRequest](docs/src/web-client/api/classes/TransactionRequest.md) - Class representing transaction requests
- [TransactionRequestBuilder](docs/src/web-client/api/classes/TransactionRequestBuilder.md) - Builder class for creating transaction requests
- [TransactionResult](docs/src/web-client/api/classes/TransactionResult.md) - Class representing transaction execution results
- [TransactionProver](docs/src/web-client/api/classes/TransactionProver.md) - Class for transaction proving
- [TransactionScript](docs/src/web-client/api/classes/TransactionScript.md) - Class for defining transaction scripts
- [NoteType](docs/src/web-client/api/enumerations/NoteType.md) - Enumeration for note types (Private/Public)
- [NoteAssets](docs/src/web-client/api/classes/NoteAssets.md) - Class for defining note assets
- [NoteMetadata](docs/src/web-client/api/classes/NoteMetadata.md) - Class for defining note metadata
- [FungibleAsset](docs/src/web-client/api/classes/FungibleAsset.md) - Class for defining fungible assets
- [Felt](docs/src/web-client/api/classes/Felt.md) - Class for working with field elements
- [FeltArray](docs/src/web-client/api/classes/FeltArray.md) - Class for working with arrays of field elements
- [NoteTag](docs/src/web-client/api/classes/NoteTag.md) - Class for defining note tags
- [NoteExecutionMode](docs/src/web-client/api/classes/NoteExecutionMode.md) - Class for defining note execution modes
- [NoteExecutionHint](docs/src/web-client/api/classes/NoteExecutionHint.md) - Class for defining note execution hints
- [OutputNotesArray](docs/src/web-client/api/classes/OutputNotesArray.md) - Class for working with arrays of output notes
- [NotesArray](docs/src/web-client/api/classes/NotesArray.md) - Class for working with arrays of notes

For a complete list of available classes and utilities, see the [SDK API Reference](docs/src/web-client/api/README.md). 
