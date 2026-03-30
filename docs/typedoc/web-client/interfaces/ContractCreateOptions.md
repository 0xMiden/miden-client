[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / ContractCreateOptions

# Interface: ContractCreateOptions

## Properties

### auth

> **auth**: `AuthSecretKey`

Auth secret key. Required.

***

### components

> **components**: `AccountComponent`[]

Pre-compiled AccountComponent instances. Required for contracts.

***

### seed

> **seed**: `Uint8Array`

Raw 32-byte seed (Uint8Array). Required.

***

### storage?

> `optional` **storage**: [`StorageMode`](../type-aliases/StorageMode.md)

Storage mode. Defaults to "public" for contracts.

***

### type?

> `optional` **type**: `2` \| `3`

Account type. Use AccountType.ImmutableContract (2) or AccountType.MutableContract (3).
