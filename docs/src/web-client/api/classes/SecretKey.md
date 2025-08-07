[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / SecretKey

# Class: SecretKey

## Methods

### free()

> **free**(): `void`

#### Returns

`void`

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

### deserialize()

> `static` **deserialize**(`bytes`): `SecretKey`

#### Parameters

##### bytes

`Uint8Array`

#### Returns

`SecretKey`

***

### withRng()

> `static` **withRng**(`seed`?): `SecretKey`

#### Parameters

##### seed?

`Uint8Array`

#### Returns

`SecretKey`
