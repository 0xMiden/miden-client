---
title: PublicKey
draft: true
---

[**@demox-labs/miden-sdk**](../index)

***

[@demox-labs/miden-sdk](../index) / PublicKey

# Class: PublicKey

## Methods

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

[`Word`](Word)

##### signature

[`Signature`](Signature)

#### Returns

`boolean`

***

### verifyData()

> **verifyData**(`signing_inputs`, `signature`): `boolean`

#### Parameters

##### signing\_inputs

[`SigningInputs`](SigningInputs)

##### signature

[`Signature`](Signature)

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
