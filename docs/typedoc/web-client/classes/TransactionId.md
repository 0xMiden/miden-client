[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / TransactionId

# Class: TransactionId

Identifier of a transaction.

## Methods

### \[dispose\]()

> **\[dispose\]**(): `void`

#### Returns

`void`

***

### asBytes()

> **asBytes**(): `Uint8Array`

Returns the transaction ID as raw bytes.

#### Returns

`Uint8Array`

***

### asElements()

> **asElements**(): [`Felt`](Felt.md)[]

Returns the transaction ID as field elements.

#### Returns

[`Felt`](Felt.md)[]

***

### free()

> **free**(): `void`

#### Returns

`void`

***

### inner()

> **inner**(): [`Word`](Word.md)

Returns the underlying word representation.

#### Returns

[`Word`](Word.md)

***

### toHex()

> **toHex**(): `string`

Returns the hexadecimal encoding of the transaction ID.

#### Returns

`string`
