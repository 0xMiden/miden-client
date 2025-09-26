---
title: SecretKey
draft: true
---

[**@demox-labs/miden-sdk**](../index)

***

[@demox-labs/miden-sdk](../index) / SecretKey

# Class: SecretKey

## Methods

### free()

> **free**(): `void`

#### Returns

`void`

***

### publicKey()

> **publicKey**(): [`PublicKey`](PublicKey)

#### Returns

[`PublicKey`](PublicKey)

***

### serialize()

> **serialize**(): `Uint8Array`

#### Returns

`Uint8Array`

***

### sign()

> **sign**(`message`): [`Signature`](Signature)

#### Parameters

##### message

[`Word`](Word)

#### Returns

[`Signature`](Signature)

***

### signData()

> **signData**(`signing_inputs`): [`Signature`](Signature)

#### Parameters

##### signing\_inputs

[`SigningInputs`](SigningInputs)

#### Returns

[`Signature`](Signature)

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

> `static` **withRng**(`seed?`): `SecretKey`

#### Parameters

##### seed?

`Uint8Array`

#### Returns

`SecretKey`
