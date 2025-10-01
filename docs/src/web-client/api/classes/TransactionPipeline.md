[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / TransactionPipeline

# Class: TransactionPipeline

WASM wrapper around the native [`TransactionPipeline`].

## Methods

### executedTransaction()

> **executedTransaction**(): [`ExecutedTransaction`](ExecutedTransaction.md)

Returns execution details after the transaction has been run.

#### Returns

[`ExecutedTransaction`](ExecutedTransaction.md)

***

### free()

> **free**(): `void`

#### Returns

`void`

***

### futureNotes()

> **futureNotes**(): [`NoteDetailsAndTag`](NoteDetailsAndTag.md)[]

Returns notes that are expected to be created as follow-up outputs.

#### Returns

[`NoteDetailsAndTag`](NoteDetailsAndTag.md)[]

***

### getTransactionUpdate()

> **getTransactionUpdate**(): [`TransactionStoreUpdate`](TransactionStoreUpdate.md)

Builds a store update using the submission height maintained by the pipeline.

#### Returns

[`TransactionStoreUpdate`](TransactionStoreUpdate.md)

***

### getTransactionUpdateWithHeight()

> **getTransactionUpdateWithHeight**(`submission_height`): [`TransactionStoreUpdate`](TransactionStoreUpdate.md)

Builds a store update using a custom submission height.

#### Parameters

##### submission\_height

`number`

#### Returns

[`TransactionStoreUpdate`](TransactionStoreUpdate.md)

***

### id()

> **id**(): [`TransactionId`](TransactionId.md)

Returns the ID of the transaction once execution succeeds.

#### Returns

[`TransactionId`](TransactionId.md)

***

### provenTransaction()

> **provenTransaction**(): [`ProvenTransaction`](ProvenTransaction.md)

Returns the proven transaction if a proof has already been generated.

#### Returns

[`ProvenTransaction`](ProvenTransaction.md)

***

### proveTransaction()

> **proveTransaction**(`prover?`): `Promise`\<[`ProvenTransaction`](ProvenTransaction.md)\>

Generates and caches a proof for the executed transaction.

#### Parameters

##### prover?

[`TransactionProver`](TransactionProver.md)

#### Returns

`Promise`\<[`ProvenTransaction`](ProvenTransaction.md)\>

***

### submitProvenTransaction()

> **submitProvenTransaction**(): `Promise`\<[`TransactionStoreUpdate`](TransactionStoreUpdate.md)\>

Submits the proven transaction and returns the resulting store update.

#### Returns

`Promise`\<[`TransactionStoreUpdate`](TransactionStoreUpdate.md)\>

***

### transactionRequest()

> **transactionRequest**(): [`TransactionRequest`](TransactionRequest.md)

Returns the pipeline's transaction request.

#### Returns

[`TransactionRequest`](TransactionRequest.md)
