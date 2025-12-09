[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / TransactionSummary

# Class: TransactionSummary

Represents a transaction summary.

## Methods

### \[dispose\]()

> **\[dispose\]**(): `void`

#### Returns

`void`

***

### accountDelta()

> **accountDelta**(): [`AccountDelta`](AccountDelta.md)

Returns the account delta described by the summary.

#### Returns

[`AccountDelta`](AccountDelta.md)

***

### free()

> **free**(): `void`

#### Returns

`void`

***

### inputNotes()

> **inputNotes**(): [`InputNotes`](InputNotes.md)

Returns the input notes referenced by the summary.

#### Returns

[`InputNotes`](InputNotes.md)

***

### outputNotes()

> **outputNotes**(): [`OutputNotes`](OutputNotes.md)

Returns the output notes referenced by the summary.

#### Returns

[`OutputNotes`](OutputNotes.md)

***

### salt()

> **salt**(): [`Word`](Word.md)

Returns the random salt mixed into the summary commitment.

#### Returns

[`Word`](Word.md)

***

### serialize()

> **serialize**(): `Uint8Array`

Serializes the summary into bytes.

#### Returns

`Uint8Array`

***

### deserialize()

> `static` **deserialize**(`bytes`): `TransactionSummary`

Deserializes a summary from bytes.

#### Parameters

##### bytes

`Uint8Array`

#### Returns

`TransactionSummary`
