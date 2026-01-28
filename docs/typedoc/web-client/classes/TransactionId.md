[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / TransactionId

# Class: TransactionId

A unique identifier of a transaction.

Transaction ID is computed as a hash of the initial and final account commitments together with
the commitments of the input and output notes.

This achieves the following properties:
- Transactions are identical if and only if they have the same ID.
- Computing transaction ID can be done solely from public transaction data.

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
