[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / TransactionRequest

# Class: TransactionRequest

Encapsulates all inputs required to execute a transaction.

## Methods

### authArg()

> **authArg**(): [`Word`](Word.md)

Returns the optional authentication argument provided to the transaction.

#### Returns

[`Word`](Word.md)

***

### expectedFutureNotes()

> **expectedFutureNotes**(): [`NoteDetailsAndTag`](NoteDetailsAndTag.md)[]

Returns future notes expected to be created by the transaction.

#### Returns

[`NoteDetailsAndTag`](NoteDetailsAndTag.md)[]

***

### expectedOutputOwnNotes()

> **expectedOutputOwnNotes**(): [`Note`](Note.md)[]

Returns notes the transaction expects to create and own.

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

Returns the optional script argument provided to the transaction.

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
