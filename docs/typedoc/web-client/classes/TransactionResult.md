[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / TransactionResult

# Class: TransactionResult

Result of executing a transaction, including notes and account deltas.

## Methods

### accountDelta()

> **accountDelta**(): [`AccountDelta`](AccountDelta.md)

Returns the resulting account delta.

#### Returns

[`AccountDelta`](AccountDelta.md)

***

### blockNum()

> **blockNum**(): `number`

Returns the block number the transaction was executed in.

#### Returns

`number`

***

### consumedNotes()

> **consumedNotes**(): [`InputNotes`](InputNotes.md)

Returns the notes consumed by the transaction.

#### Returns

[`InputNotes`](InputNotes.md)

***

### createdNotes()

> **createdNotes**(): [`OutputNotes`](OutputNotes.md)

Returns notes created by the transaction.

#### Returns

[`OutputNotes`](OutputNotes.md)

***

### executedTransaction()

> **executedTransaction**(): [`ExecutedTransaction`](ExecutedTransaction.md)

Returns the executed transaction details.

#### Returns

[`ExecutedTransaction`](ExecutedTransaction.md)

***

### free()

> **free**(): `void`

#### Returns

`void`

***

### serialize()

> **serialize**(): `Uint8Array`

Serializes the transaction result into bytes.

#### Returns

`Uint8Array`

***

### transactionArguments()

> **transactionArguments**(): [`TransactionArgs`](TransactionArgs.md)

Returns the arguments consumed by the transaction script.

#### Returns

[`TransactionArgs`](TransactionArgs.md)

***

### deserialize()

> `static` **deserialize**(`bytes`): `TransactionResult`

Deserializes a transaction result from bytes.

#### Parameters

##### bytes

`Uint8Array`

#### Returns

`TransactionResult`
