---
title: AccountComponent
draft: true
---

[**@demox-labs/miden-sdk**](../index)

***

[@demox-labs/miden-sdk](../index) / AccountComponent

# Class: AccountComponent

## Methods

### free()

> **free**(): `void`

#### Returns

`void`

***

### getProcedureHash()

> **getProcedureHash**(`procedure_name`): `string`

#### Parameters

##### procedure\_name

`string`

#### Returns

`string`

***

### withSupportsAllTypes()

> **withSupportsAllTypes**(): `AccountComponent`

#### Returns

`AccountComponent`

***

### compile()

> `static` **compile**(`account_code`, `assembler`, `storage_slots`): `AccountComponent`

#### Parameters

##### account\_code

`string`

##### assembler

[`Assembler`](Assembler)

##### storage\_slots

[`StorageSlot`](StorageSlot)[]

#### Returns

`AccountComponent`

***

### createAuthComponent()

> `static` **createAuthComponent**(`secret_key`): `AccountComponent`

#### Parameters

##### secret\_key

[`SecretKey`](SecretKey)

#### Returns

`AccountComponent`
