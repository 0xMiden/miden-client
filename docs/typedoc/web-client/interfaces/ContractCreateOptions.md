[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / ContractCreateOptions

# Interface: ContractCreateOptions

Options for creating a custom contract account.

Unlike wallets/faucets, `auth` must be a raw `AuthSecretKey` WASM object —
the caller must retain it for signing. Construct via `AuthSecretKey.rpoFalconWithRNG(seed)`.
Storage defaults to `"public"` (unlike wallets which default to `"private"`).

## Properties

### auth

> **auth**: `AuthSecretKey`

Required raw WASM AuthSecretKey. Use `AuthSecretKey.rpoFalconWithRNG(seed)`.
Must be a concrete object (not a string) because the caller needs to retain
the key for transaction signing.

***

### components?

> `optional` **components**: `AccountComponent`[]

Additional compiled account components from `compile.component()`.

***

### seed

> **seed**: `Uint8Array`

Required — used to derive a deterministic account ID.

***

### storage?

> `optional` **storage**: [`StorageMode`](../type-aliases/StorageMode.md)

Defaults to "public" (differs from wallet default of "private").

***

### type

> **type**: `"ImmutableContract"` \| `"MutableContract"`
