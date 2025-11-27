---
title: New-transactions
sidebar_position: 8
---

# Creating Transactions with the Miden SDK

This guide demonstrates how to create and submit different types of transactions using the Miden SDK. We'll cover minting, sending, consuming, and custom transactions.

## Basic Transaction Flow

All transactions follow a similar pattern:
1. Create a transaction request
2. Execute the transaction to perform local validation and execution
3. Prove the transaction (locally or by using a remote prover)
4. Submit the proven transaction to the network and apply the resulting update

If you don't need to inspect the transaction intermediate structures manually, the SDK offers `submitNewTransaction` to run steps 2-4 for you:

```typescript
import { NoteType, WebClient } from "@demox-labs/miden-sdk";

try {
    const webClient = await WebClient.createClient();
    // 1. Create a transaction request
    const transactionRequest = webClient.newMintTransactionRequest(
        recipientAccountId, // Account that will receive the minted tokens
        faucetAccountId,    // Faucet account that mints the tokens
        NoteType.Private,
        1000
    );
    // 2-4. Execute transaction, prove and submit to the network
    const transactionId = await webClient.submitNewTransaction(
        faucetAccountId,
        transactionRequest
    );

    console.log("Submitted transaction:", transactionId.toString());

    await webClient.syncState();
    const consumableNotes = await webClient.getConsumableNotes(recipientAccountId);
    console.log("Minted note ID:", consumableNotes[0].inputNoteRecord().id().toString());
} catch (error) {
    console.error("Mint transaction failed:", error.message);
}
```

When you need to inspect execution results before proving, fall back to the manual pipeline:

```typescript
import { NoteType, TransactionProver, WebClient } from "@demox-labs/miden-sdk";

try {
    const webClient = await WebClient.createClient();

    // 1. Create a transaction request
    const transactionRequest = webClient.newMintTransactionRequest(
        recipientAccountId,
        faucetAccountId,
        NoteType.Private,
        1000
    );

    // 2. Execute the transaction to perform local validation and execution
    const result = await webClient.executeTransaction(
        faucetAccountId,
        transactionRequest
    );

    const executedTx = result.executedTransaction();
    console.log("Created notes:", executedTx.outputNotes());
    console.log("Consumed notes:", executedTx.inputNotes());
    console.log("Account delta:", executedTx.accountDelta());

    // 3. Prove the transaction (locally or by using a remote prover)
    const proven = await webClient.proveTransaction(
        result,
        TransactionProver.newLocalProver()
    );

    // 4. Submit the proven transaction to the network and apply the resulting update
    const submissionHeight = await webClient.submitProvenTransaction(
        proven,
        result
    );
    const transactionUpdate = await webClient.applyTransaction(
        result,
        submissionHeight
    );

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

    const result = await webClient.executeTransaction(
        accountId,
        transactionRequest
    );

    const proven = await webClient.proveTransaction(result, remoteProver);
    const submissionHeight = await webClient.submitProvenTransaction(
        proven,
        result
    );
    const transactionUpdate = await webClient.applyTransaction(
        result,
        submissionHeight
    );

    console.log("Block number:", transactionUpdate.blockNum());
    console.log(
        "Submitted transaction:",
        transactionUpdate.executedTransaction().id().toHex()
    );
} catch (error) {
    console.error("Transaction failed:", error.message);
}
```

:::note
Using a remote prover can significantly improve performance for complex transactions by offloading the computationally intensive proving work to a dedicated server. This is particularly useful when dealing with large transactions or when running in resource-constrained environments.
:::

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
        faucetId,         // Faucet account ID
        NoteType.Private, // Note type
        100,              // Amount to send
        100,              // Optional recall height
        90                // Optional timelock height
    );

    const result = await webClient.executeTransaction(
        senderAccountId,
        transactionRequest
    );

    const proven = await webClient.proveTransaction(
        result,
        TransactionProver.newLocalProver()
    );
    const submissionHeight = await webClient.submitProvenTransaction(
        proven,
        result
    );
    const transactionUpdate = await webClient.applyTransaction(
        result,
        submissionHeight
    );

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

    const result = await webClient.executeTransaction(
        accountId,
        transactionRequest
    );

    const proven = await webClient.proveTransaction(
        result,
        TransactionProver.newLocalProver()
    );
    const submissionHeight = await webClient.submitProvenTransaction(
        proven,
        result
    );
    const transactionUpdate = await webClient.applyTransaction(
        result,
        submissionHeight
    );

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

:::note
For a complete example of a custom transaction implementation, including input notes, output notes, and custom scripts, see the integration tests in [`new_transactions.test.ts`](https://github.com/0xMiden/miden-client/blob/main/crates/web-client/test/new_transactions.test.ts).
:::

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
    OutputNoteArray,
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
    noteArgs.forEach(felt => feltArray.push(felt));

    // Create custom note script
    const noteScript = `
        # Your custom note script here
        # This can include custom validation logic, asset transfers, etc.
    `;

    // Create transaction script
    const transactionScript = new TransactionScript(noteScript);

    // Create output notes array
    const outputNotes = new OutputNoteArray();
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
    const result = await webClient.executeTransaction(
        accountId,
        transactionRequest
    );

    const proven = await webClient.proveTransaction(
        result,
        TransactionProver.newLocalProver()
    );
    const submissionHeight = await webClient.submitProvenTransaction(
        proven,
        result
    );
    const transactionUpdate = await webClient.applyTransaction(
        result,
        submissionHeight
    );

    // Access transaction details
    console.log("Block number:", transactionUpdate.blockNum());
    console.log("Created notes:", transactionUpdate.executedTransaction().outputNotes());
    console.log("Consumed notes:", transactionUpdate.executedTransaction().inputNotes());
    console.log("Account delta:", transactionUpdate.executedTransaction().accountDelta());
} catch (error) {
    console.error("Custom transaction failed:", error.message);
}
```

:::note
Custom transactions require a good understanding of the Miden VM and its instruction set. They are powerful but should be used with caution as they can affect the security and correctness of your application.
:::

## Relevant Documentation

For more detailed information about transaction functionality, refer to the following API documentation:

- [WebClient](https://github.com/0xMiden/miden-client/docs/typedoc/web-client/classes/WebClient.md) - Main client class for transaction operations
- [TransactionRequest](https://github.com/0xMiden/miden-client/docs/typedoc/web-client/classes/TransactionRequest.md) - Class representing transaction requests
- [TransactionRequestBuilder](https://github.com/0xMiden/miden-client/docs/typedoc/web-client/classes/TransactionRequestBuilder.md) - Builder class for creating transaction requests
- [TransactionResult](https://github.com/0xMiden/miden-client/docs/typedoc/web-client/classes/TransactionResult.md) - Class representing transaction execution results
- [TransactionProver](https://github.com/0xMiden/miden-client/docs/typedoc/web-client/classes/TransactionProver.md) - Class for transaction proving
- [TransactionScript](https://github.com/0xMiden/miden-client/docs/typedoc/web-client/classes/TransactionScript.md) - Class for defining transaction scripts
- [NoteType](https://github.com/0xMiden/miden-client/docs/typedoc/web-client/enumerations/NoteType.md) - Enumeration for note types (Private/Public)
- [NoteAssets](https://github.com/0xMiden/miden-client/docs/typedoc/web-client/classes/NoteAssets.md) - Class for defining note assets
- [NoteMetadata](https://github.com/0xMiden/miden-client/docs/typedoc/web-client/classes/NoteMetadata.md) - Class for defining note metadata
- [FungibleAsset](https://github.com/0xMiden/miden-client/docs/typedoc/web-client/classes/FungibleAsset.md) - Class for defining fungible assets
- [Felt](https://github.com/0xMiden/miden-client/docs/typedoc/web-client/classes/Felt.md) - Class for working with field elements
- [FeltArray](https://github.com/0xMiden/miden-client/docs/typedoc/web-client/classes/FeltArray.md) - Class for working with arrays of field elements
- [NoteTag](https://github.com/0xMiden/miden-client/docs/typedoc/web-client/classes/NoteTag.md) - Class for defining note tags
- [NoteExecutionMode](https://github.com/0xMiden/miden-client/docs/typedoc/web-client/classes/NoteExecutionMode.md) - Class for defining note execution modes
- [NoteExecutionHint](https://github.com/0xMiden/miden-client/docs/typedoc/web-client/classes/NoteExecutionHint.md) - Class for defining note execution hints
- [OutputNotesArray](https://github.com/0xMiden/miden-client/docs/typedoc/web-client/classes/OutputNotesArray.md) - Class for working with arrays of output notes
- [NotesArray](https://github.com/0xMiden/miden-client/docs/typedoc/web-client/classes/NotesArray.md) - Class for working with arrays of notes

For a complete list of available classes and utilities, see the [SDK API Reference](https://github.com/0xMiden/miden-client/docs/typedoc/web-client/README.md).
