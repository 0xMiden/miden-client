[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / TransactionRequest

# Class: TransactionRequest

## Methods

### \[dispose\]()

> **\[dispose\]**(): `void`

#### Returns

`void`

***

### authArg()

> **authArg**(): [`Word`](Word.md)

Returns the authentication argument if present.

#### Returns

[`Word`](Word.md)

***

### expectedFutureNotes()

> **expectedFutureNotes**(): [`NoteDetailsAndTag`](NoteDetailsAndTag.md)[]

Returns notes expected to be created in subsequent executions.

#### Returns

[`NoteDetailsAndTag`](NoteDetailsAndTag.md)[]

***

### expectedOutputOwnNotes()

> **expectedOutputOwnNotes**(): [`Note`](Note.md)[]

Returns output notes created by the sender account.

#### Returns

[`Note`](Note.md)[]

***

### free()

> **free**(): `void`

#### Returns

`void`

***

### scriptArg()

> **scriptArg**(): [`Word`](Word.md)

Returns the transaction script argument if present.

#### Returns

[`Word`](Word.md)

***

### serialize()

> **serialize**(): `Uint8Array`

Serializes the transaction request into bytes.

#### Returns

`Uint8Array`

***

### deserialize()

> `static` **deserialize**(`bytes`): `TransactionRequest`

Deserializes a transaction request from bytes.

#### Parameters

##### bytes

`Uint8Array`

#### Returns

`TransactionRequest`
