[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / TransactionSummary

# Class: TransactionSummary

Represents a transaction summary.

## Methods

### \[dispose\]()

> **\[dispose\]**(): `void`

#### Returns

`void`

***

### accountDelta()

> **accountDelta**(): `AccountDelta`

Returns the account delta described by the summary.

#### Returns

`AccountDelta`

***

### free()

> **free**(): `void`

#### Returns

`void`

***

### inputNotes()

> **inputNotes**(): `InputNotes`

Returns the input notes referenced by the summary.

#### Returns

`InputNotes`

***

### outputNotes()

> **outputNotes**(): `OutputNotes`

Returns the output notes referenced by the summary.

#### Returns

`OutputNotes`

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

### toCommitment()

> **toCommitment**(): [`Word`](Word.md)

Computes the commitment to this `TransactionSummary`.

#### Returns

[`Word`](Word.md)

***

### deserialize()

> `static` **deserialize**(`bytes`): `TransactionSummary`

Deserializes a summary from bytes.

#### Parameters

##### bytes

`Uint8Array`

#### Returns

`TransactionSummary`
