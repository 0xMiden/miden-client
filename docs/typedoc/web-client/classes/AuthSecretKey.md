[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / AuthSecretKey

# Class: AuthSecretKey

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

### getEcdsaK256KeccakSecretKeyAsFelts()

> **getEcdsaK256KeccakSecretKeyAsFelts**(): [`Felt`](Felt.md)[]

Returns the ECDSA k256 Keccak secret key bytes encoded as felts.

#### Returns

[`Felt`](Felt.md)[]

***

### getPublicKeyAsWord()

> **getPublicKeyAsWord**(): [`Word`](Word.md)

#### Returns

[`Word`](Word.md)

***

### getRpoFalcon512SecretKeyAsFelts()

> **getRpoFalcon512SecretKeyAsFelts**(): [`Felt`](Felt.md)[]

#### Returns

[`Felt`](Felt.md)[]

***

### publicKey()

> **publicKey**(): [`PublicKey`](PublicKey.md)

#### Returns

[`PublicKey`](PublicKey.md)

***

### serialize()

> **serialize**(): `Uint8Array`

#### Returns

`Uint8Array`

***

### sign()

> **sign**(`message`): [`Signature`](Signature.md)

#### Parameters

##### message

[`Word`](Word.md)

#### Returns

[`Signature`](Signature.md)

***

### signData()

> **signData**(`signing_inputs`): [`Signature`](Signature.md)

#### Parameters

##### signing\_inputs

[`SigningInputs`](SigningInputs.md)

#### Returns

[`Signature`](Signature.md)

***

### deserialize()

> `static` **deserialize**(`bytes`): `AuthSecretKey`

#### Parameters

##### bytes

`Uint8Array`

#### Returns

`AuthSecretKey`

***

### ecdsaWithRNG()

> `static` **ecdsaWithRNG**(`seed?`): `AuthSecretKey`

#### Parameters

##### seed?

`Uint8Array`

#### Returns

`AuthSecretKey`

***

### rpoFalconWithRNG()

> `static` **rpoFalconWithRNG**(`seed?`): `AuthSecretKey`

#### Parameters

##### seed?

`Uint8Array`

#### Returns

`AuthSecretKey`
