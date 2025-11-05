[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / SecretKey

# Class: SecretKey

Secret key capable of producing RPO Falcon signatures.

## Methods

### free()

> **free**(): `void`

#### Returns

`void`

***

### publicKey()

> **publicKey**(): [`PublicKey`](PublicKey.md)

Returns the public key corresponding to this secret key.

#### Returns

[`PublicKey`](PublicKey.md)

***

### serialize()

> **serialize**(): `Uint8Array`

Serializes the secret key into bytes.

#### Returns

`Uint8Array`

***

### sign()

> **sign**(`message`): [`Signature`](Signature.md)

Signs a simple message commitment and returns the signature.

#### Parameters

##### message

[`Word`](Word.md)

#### Returns

[`Signature`](Signature.md)

***

### signData()

> **signData**(`signing_inputs`): [`Signature`](Signature.md)

Signs the provided signing inputs and returns the resulting signature.

#### Parameters

##### signing\_inputs

[`SigningInputs`](SigningInputs.md)

#### Returns

[`Signature`](Signature.md)

***

### deserialize()

> `static` **deserialize**(`bytes`): `SecretKey`

Deserializes a secret key from bytes.

#### Parameters

##### bytes

`Uint8Array`

#### Returns

`SecretKey`

***

### withRng()

> `static` **withRng**(`seed?`): `SecretKey`

Generates a new secret key using an optional RNG seed.

#### Parameters

##### seed?

`Uint8Array`

#### Returns

`SecretKey`
