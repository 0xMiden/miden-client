[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / TransactionStoreUpdate

# Class: TransactionStoreUpdate

Update describing the effects of a stored transaction.

## Methods

### \[dispose\]()

> **\[dispose\]**(): `void`

#### Returns

`void`

***

### accountDelta()

> **accountDelta**(): [`AccountDelta`](AccountDelta.md)

Returns the account delta applied by the transaction.

#### Returns

[`AccountDelta`](AccountDelta.md)

***

### createdNotes()

> **createdNotes**(): [`OutputNotes`](OutputNotes.md)

Returns the notes created by the transaction.

#### Returns

[`OutputNotes`](OutputNotes.md)

***

### executedTransaction()

> **executedTransaction**(): [`ExecutedTransaction`](ExecutedTransaction.md)

Returns the executed transaction associated with this update.

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

Returns notes expected to be created in follow-up executions.

#### Returns

[`NoteDetailsAndTag`](NoteDetailsAndTag.md)[]

***

### serialize()

> **serialize**(): `Uint8Array`

Serializes the update into bytes.

#### Returns

`Uint8Array`

***

### submissionHeight()

> **submissionHeight**(): `number`

Returns the block height at which the transaction was submitted.

#### Returns

`number`

***

### deserialize()

> `static` **deserialize**(`bytes`): `TransactionStoreUpdate`

Deserializes an update from bytes.

#### Parameters

##### bytes

`Uint8Array`

#### Returns

`TransactionStoreUpdate`
