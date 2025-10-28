[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / TransactionResult

# Class: TransactionResult

WASM wrapper around the native [`TransactionResult`].

## Methods

### executedTransaction()

> **executedTransaction**(): [`ExecutedTransaction`](ExecutedTransaction.md)

Returns the executed transaction.

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

Returns notes that are expected to be created as a result of follow-up executions.

#### Returns

[`NoteDetailsAndTag`](NoteDetailsAndTag.md)[]

***

### id()

> **id**(): [`TransactionId`](TransactionId.md)

Returns the ID of the transaction.

#### Returns

[`TransactionId`](TransactionId.md)

***

### transactionUpdateWithHeight()

> **transactionUpdateWithHeight**(`submission_height`): [`TransactionStoreUpdate`](TransactionStoreUpdate.md)

Builds a store update using the provided submission height.

#### Parameters

##### submission\_height

`number`

#### Returns

[`TransactionStoreUpdate`](TransactionStoreUpdate.md)
