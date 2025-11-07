[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / PublicKey

# Class: PublicKey

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

#### Returns

`Uint8Array`

***

### verify()

> **verify**(`message`, `signature`): `boolean`

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

#### Parameters

##### bytes

`Uint8Array`

#### Returns

`PublicKey`
