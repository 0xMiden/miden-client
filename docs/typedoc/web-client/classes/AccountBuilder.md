[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / AccountBuilder

# Class: AccountBuilder

## Constructors

### Constructor

> **new AccountBuilder**(`init_seed`): `AccountBuilder`

Creates a new account builder from a 32-byte initial seed.

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

Sets the account type (regular, faucet, etc.).

#### Parameters

##### account\_type

`AccountType`

#### Returns

`AccountBuilder`

***

### build()

> **build**(): [`AccountBuilderResult`](AccountBuilderResult.md)

Builds the account and returns it together with the derived seed.

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

Sets the storage mode (public/private) for the account.

#### Parameters

##### storage\_mode

[`AccountStorageMode`](AccountStorageMode.md)

#### Returns

`AccountBuilder`

***

### withAuthComponent()

> **withAuthComponent**(`account_component`): `AccountBuilder`

Adds an authentication component to the account.

#### Parameters

##### account\_component

[`AccountComponent`](AccountComponent.md)

#### Returns

`AccountBuilder`

***

### withBasicWalletComponent()

> **withBasicWalletComponent**(): `AccountBuilder`

#### Returns

`AccountBuilder`

***

### withComponent()

> **withComponent**(`account_component`): `AccountBuilder`

Adds a component to the account.

#### Parameters

##### account\_component

[`AccountComponent`](AccountComponent.md)

#### Returns

`AccountBuilder`

***

### withNoAuthComponent()

> **withNoAuthComponent**(): `AccountBuilder`

Adds a no-auth component to the account (for public accounts).

#### Returns

`AccountBuilder`
