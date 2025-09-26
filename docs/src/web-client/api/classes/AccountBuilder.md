---
title: AccountBuilder
draft: true
---

[**@demox-labs/miden-sdk**](../index)

***

[@demox-labs/miden-sdk](../index) / AccountBuilder

# Class: AccountBuilder

## Constructors

### Constructor

> **new AccountBuilder**(`init_seed`): `AccountBuilder`

#### Parameters

##### init\_seed

`Uint8Array`

#### Returns

`AccountBuilder`

## Methods

### accountType()

> **accountType**(`account_type`): `AccountBuilder`

#### Parameters

##### account\_type

[`AccountType`](../enumerations/AccountType)

#### Returns

`AccountBuilder`

***

### build()

> **build**(): [`AccountBuilderResult`](AccountBuilderResult)

#### Returns

[`AccountBuilderResult`](AccountBuilderResult)

***

### free()

> **free**(): `void`

#### Returns

`void`

***

### storageMode()

> **storageMode**(`storage_mode`): `AccountBuilder`

#### Parameters

##### storage\_mode

[`AccountStorageMode`](AccountStorageMode)

#### Returns

`AccountBuilder`

***

### withAuthComponent()

> **withAuthComponent**(`account_component`): `AccountBuilder`

#### Parameters

##### account\_component

[`AccountComponent`](AccountComponent)

#### Returns

`AccountBuilder`

***

### withComponent()

> **withComponent**(`account_component`): `AccountBuilder`

#### Parameters

##### account\_component

[`AccountComponent`](AccountComponent)

#### Returns

`AccountBuilder`
