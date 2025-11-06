[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / TransactionResult

# Class: TransactionResult

WASM wrapper around the native [`TransactionResult`].

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
