[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / AccountBuilder

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

### \[dispose\]()

> **\[dispose\]**(): `void`

#### Returns

`void`

***

### accountType()

> **accountType**(`account_type`): `AccountBuilder`

#### Parameters

##### account\_type

[`AccountType`](../enumerations/AccountType.md)

#### Returns

`AccountBuilder`

***

### build()

> **build**(): [`AccountBuilderResult`](AccountBuilderResult.md)

#### Returns

[`AccountBuilderResult`](AccountBuilderResult.md)

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

[`AccountStorageMode`](AccountStorageMode.md)

#### Returns

`AccountBuilder`

***

### withAuthComponent()

> **withAuthComponent**(`account_component`): `AccountBuilder`

#### Parameters

##### account\_component

[`AccountComponent`](AccountComponent.md)

#### Returns

`AccountBuilder`

***

### withComponent()

> **withComponent**(`account_component`): `AccountBuilder`

#### Parameters

##### account\_component

[`AccountComponent`](AccountComponent.md)

#### Returns

`AccountBuilder`

***

### withNoAuthComponent()

> **withNoAuthComponent**(): `AccountBuilder`

#### Returns

`AccountBuilder`
