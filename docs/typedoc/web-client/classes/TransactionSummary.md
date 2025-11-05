[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / TransactionSummary

# Class: TransactionSummary

Summary of a transaction used when requesting signatures.

## Methods

### accountDelta()

> **accountDelta**(): [`AccountDelta`](AccountDelta.md)

Returns the account delta captured in the summary.

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

Returns the input notes included in the summary.

#### Returns

[`InputNotes`](InputNotes.md)

***

### outputNotes()

> **outputNotes**(): [`OutputNotes`](OutputNotes.md)

Returns the expected output notes included in the summary.

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

Serializes the transaction summary into bytes.

#### Returns

`Uint8Array`

***

### deserialize()

> `static` **deserialize**(`bytes`): `TransactionSummary`

Deserializes a transaction summary from bytes.

#### Parameters

##### bytes

`Uint8Array`

#### Returns

`TransactionSummary`
