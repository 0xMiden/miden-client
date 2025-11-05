[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / PublicKey

# Class: PublicKey

Public key used for RPO Falcon signatures.

## Methods

### free()

> **free**(): `void`

#### Returns

`void`

***

### serialize()

> **serialize**(): `Uint8Array`

Serializes the public key into bytes.

#### Returns

`Uint8Array`

***

### verify()

> **verify**(`message`, `signature`): `boolean`

Verifies a signature over a simple message commitment.

#### Parameters

##### message

[`Word`](Word.md)

##### signature

[`Signature`](Signature.md)

#### Returns

`boolean`

***

### verifyData()

> **verifyData**(`signing_inputs`, `signature`): `boolean`

Verifies a signature over arbitrary signing inputs.

#### Parameters

##### signing\_inputs

[`SigningInputs`](SigningInputs.md)

##### signature

[`Signature`](Signature.md)

#### Returns

`boolean`

***

### deserialize()

> `static` **deserialize**(`bytes`): `PublicKey`

Deserializes a public key from bytes.

#### Parameters

##### bytes

`Uint8Array`

#### Returns

`PublicKey`
