[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / ContractCreateOptions

# Interface: ContractCreateOptions

## Properties

### auth

> **auth**: `AuthSecretKey`

Auth secret key. Required.

***

### components?

> `optional` **components**: `AccountComponent`[]

Pre-compiled AccountComponent instances.

***

### seed

> **seed**: `Uint8Array`

Raw 32-byte seed (Uint8Array). Required.

***

### storage?

> `optional` **storage**: [`StorageMode`](../type-aliases/StorageMode.md)

Storage mode. Defaults to "public" for contracts.

***

### type

> **type**: `"ImmutableContract"` \| `"MutableContract"`
