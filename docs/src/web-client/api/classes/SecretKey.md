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

### sign()

> **sign**(`message`): [`Signature`](Signature.md)

#### Parameters

##### message

[`Word`](Word.md)

#### Returns

[`Signature`](Signature.md)

***

### toHex()

> **toHex**(): `string`

#### Returns

`string`

***

### fromHex()

> `static` **fromHex**(`hex`): `SecretKey`

#### Parameters

##### hex

`string`

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
