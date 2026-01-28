[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / Signature

# Class: Signature

Cryptographic signature produced by supported auth schemes.

## Methods

### \[dispose\]()

> **\[dispose\]**(): `void`

#### Returns

`void`

***

### free()

> **free**(): `void`

#### Returns

`void`

***

### serialize()

> **serialize**(): `Uint8Array`

Serializes the signature into bytes.

#### Returns

`Uint8Array`

***

### toPreparedSignature()

> **toPreparedSignature**(`message`): [`Felt`](Felt.md)[]

Converts the signature to the prepared field elements expected by verifying code.

#### Parameters

##### message

[`Word`](Word.md)

#### Returns

[`Felt`](Felt.md)[]

***

### deserialize()

> `static` **deserialize**(`bytes`): `Signature`

Deserializes a signature from bytes.

#### Parameters

##### bytes

`Uint8Array`

#### Returns

`Signature`
