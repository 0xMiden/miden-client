[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / AccountBuilder

# Class: AccountBuilder

JavaScript wrapper around [`miden_client::account::AccountBuilder`].

Provides a builder interface for configuring and creating new accounts in the browser.

## Constructors

### Constructor

> **new AccountBuilder**(`init_seed`): `AccountBuilder`

Creates a new account builder from a 32-byte seed.

#### Parameters

##### init\_seed

`Uint8Array`

Seed bytes; must be exactly 32 bytes.

#### Returns

`AccountBuilder`

#### Throws

Throws if the seed length is invalid.

## Methods

### accountType()

> **accountType**(`account_type`): `AccountBuilder`

Sets the account type for the builder.

#### Parameters

##### account\_type

[`AccountType`](../enumerations/AccountType.md)

#### Returns

`AccountBuilder`

***

### build()

> **build**(): [`AccountBuilderResult`](AccountBuilderResult.md)

Builds the account using the accumulated configuration.

#### Returns

[`AccountBuilderResult`](AccountBuilderResult.md)

#### Throws

Throws if the underlying account creation fails.

***

### free()

> **free**(): `void`

#### Returns

`void`

***

### storageMode()

> **storageMode**(`storage_mode`): `AccountBuilder`

Sets the account storage mode (e.g. private or network).

#### Parameters

##### storage\_mode

[`AccountStorageMode`](AccountStorageMode.md)

#### Returns

`AccountBuilder`

***

### withAuthComponent()

> **withAuthComponent**(`account_component`): `AccountBuilder`

Sets the authentication component to use for the account.

#### Parameters

##### account\_component

[`AccountComponent`](AccountComponent.md)

#### Returns

`AccountBuilder`

***

### withComponent()

> **withComponent**(`account_component`): `AccountBuilder`

Adds an additional account component to the builder.

#### Parameters

##### account\_component

[`AccountComponent`](AccountComponent.md)

#### Returns

`AccountBuilder`

***

### withNoAuthComponent()

> **withNoAuthComponent**(): `AccountBuilder`

Configures the account to use the built-in no-auth component.

#### Returns

`AccountBuilder`
