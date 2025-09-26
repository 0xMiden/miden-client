---
title: AccountDelta
draft: true
---

[**@demox-labs/miden-sdk**](../index)

***

[@demox-labs/miden-sdk](../index) / AccountDelta

# Class: AccountDelta

## Methods

### free()

> **free**(): `void`

#### Returns

`void`

***

### id()

> **id**(): [`AccountId`](AccountId)

#### Returns

[`AccountId`](AccountId)

***

### isEmpty()

> **isEmpty**(): `boolean`

#### Returns

`boolean`

***

### nonceDelta()

> **nonceDelta**(): [`Felt`](Felt)

#### Returns

[`Felt`](Felt)

***

### serialize()

> **serialize**(): `Uint8Array`

#### Returns

`Uint8Array`

***

### storage()

> **storage**(): [`AccountStorageDelta`](AccountStorageDelta)

#### Returns

[`AccountStorageDelta`](AccountStorageDelta)

***

### vault()

> **vault**(): [`AccountVaultDelta`](AccountVaultDelta)

#### Returns

[`AccountVaultDelta`](AccountVaultDelta)

***

### deserialize()

> `static` **deserialize**(`bytes`): `AccountDelta`

#### Parameters

##### bytes

`Uint8Array`

#### Returns

`AccountDelta`
