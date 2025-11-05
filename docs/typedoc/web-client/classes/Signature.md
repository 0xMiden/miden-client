[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / Signature

# Class: Signature

Cryptographic signature produced by the Miden authentication scheme.

## Methods

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

> **toPreparedSignature**(): [`Felt`](Felt.md)[]

Returns the pre-processed signature elements expected by the verifier circuit.

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
