[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / TransactionResult

# Class: TransactionResult

Represents the result of executing a transaction by the client.

It contains an `ExecutedTransaction`, and a list of `future_notes`
that we expect to receive in the future (you can check at swap notes for an example of this).

## Methods

### \[dispose\]()

> **\[dispose\]**(): `void`

#### Returns

`void`

***

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

### serialize()

> **serialize**(): `Uint8Array`

Serializes the transaction result into bytes.

#### Returns

`Uint8Array`

***

### deserialize()

> `static` **deserialize**(`bytes`): `TransactionResult`

Deserializes a transaction result from bytes.

#### Parameters

##### bytes

`Uint8Array`

#### Returns

`TransactionResult`
