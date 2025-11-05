[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / AccountBuilderResult

# Class: AccountBuilderResult

Result of constructing a new account via [`AccountBuilder`].

Exposes the built account and the seed used to derive it so values can be persisted on the
JavaScript side.

## Properties

### account

> `readonly` **account**: [`Account`](Account.md)

Returns the newly built account instance.

***

### seed

> `readonly` **seed**: [`Word`](Word.md)

Returns the seed used to derive the account keys.

## Methods

### free()

> **free**(): `void`

#### Returns

`void`
