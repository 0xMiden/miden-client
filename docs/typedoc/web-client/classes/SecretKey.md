[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / SecretKey

# Class: SecretKey

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

### publicKey()

> **publicKey**(): [`PublicKey`](PublicKey.md)

Returns the public key associated with this secret key.

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

Signs a message word (blind signature).

#### Parameters

##### message

[`Word`](Word.md)

#### Returns

[`Signature`](Signature.md)

***

### signData()

> **signData**(`signing_inputs`): [`Signature`](Signature.md)

Signs arbitrary signing inputs.

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

### ecdsaWithRNG()

> `static` **ecdsaWithRNG**(`seed?`): `SecretKey`

Generates an ECDSA k256 Keccak secret key using an optional deterministic seed.

#### Parameters

##### seed?

`Uint8Array`\<`ArrayBufferLike`\>

#### Returns

`SecretKey`

***

### rpoFalconWithRNG()

> `static` **rpoFalconWithRNG**(`seed?`): `SecretKey`

Generates an `RpoFalcon512` secret key using an optional deterministic seed.

#### Parameters

##### seed?

`Uint8Array`\<`ArrayBufferLike`\>

#### Returns

`SecretKey`
