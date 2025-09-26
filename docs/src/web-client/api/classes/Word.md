---
title: Word
draft: true
---

[**@demox-labs/miden-sdk**](../index)

***

[@demox-labs/miden-sdk](../index) / Word

# Class: Word

## Constructors

### Constructor

> **new Word**(`u64_vec`): `Word`

#### Parameters

##### u64\_vec

`BigUint64Array`

#### Returns

`Word`

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

### toFelts()

> **toFelts**(): [`Felt`](Felt.md)[]

#### Returns

[`Felt`](Felt.md)[]

***

### toHex()

> **toHex**(): `string`

#### Returns

`string`

***

### toU64s()

> **toU64s**(): `BigUint64Array`

#### Returns

`BigUint64Array`

***

### deserialize()

> `static` **deserialize**(`bytes`): `Word`

#### Parameters

##### bytes

`Uint8Array`

#### Returns

`Word`

***

### fromHex()

> `static` **fromHex**(`hex`): `Word`

Creates a Word from a hex string.
Fails if the provided string is not a valid hex representation of a Word.

#### Parameters

##### hex

`string`

#### Returns

`Word`

***

### newFromFelts()

> `static` **newFromFelts**(`felt_vec`): `Word`

#### Parameters

##### felt\_vec

[`Felt`](Felt.md)[]

#### Returns

`Word`
