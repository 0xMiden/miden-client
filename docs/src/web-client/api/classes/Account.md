---
title: Account
draft: true
---

[**@demox-labs/miden-sdk**](../index)

***

[@demox-labs/miden-sdk](../index) / Account

# Class: Account

## Methods

### code()

> **code**(): [`AccountCode`](AccountCode)

#### Returns

[`AccountCode`](AccountCode)

***

### commitment()

> **commitment**(): [`Word`](Word)

#### Returns

[`Word`](Word)

***

### free()

> **free**(): `void`

#### Returns

`void`

***

### getPublicKeys()

> **getPublicKeys**(): [`Word`](Word)[]

#### Returns

[`Word`](Word)[]

***

### id()

> **id**(): [`AccountId`](AccountId)

#### Returns

[`AccountId`](AccountId)

***

### isFaucet()

> **isFaucet**(): `boolean`

#### Returns

`boolean`

***

### isNew()

> **isNew**(): `boolean`

#### Returns

`boolean`

***

### isPublic()

> **isPublic**(): `boolean`

#### Returns

`boolean`

***

### isRegularAccount()

> **isRegularAccount**(): `boolean`

#### Returns

`boolean`

***

### isUpdatable()

> **isUpdatable**(): `boolean`

#### Returns

`boolean`

***

### nonce()

> **nonce**(): [`Felt`](Felt)

#### Returns

[`Felt`](Felt)

***

### serialize()

> **serialize**(): `Uint8Array`

#### Returns

`Uint8Array`

***

### storage()

> **storage**(): [`AccountStorage`](AccountStorage)

#### Returns

[`AccountStorage`](AccountStorage)

***

### vault()

> **vault**(): [`AssetVault`](AssetVault)

#### Returns

[`AssetVault`](AssetVault)

***

### deserialize()

> `static` **deserialize**(`bytes`): `Account`

#### Parameters

##### bytes

`Uint8Array`

#### Returns

`Account`
