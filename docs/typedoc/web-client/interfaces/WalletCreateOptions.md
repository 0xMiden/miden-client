[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / WalletCreateOptions

# Interface: WalletCreateOptions

## Properties

### auth?

> `optional` **auth**: [`AuthSchemeType`](../type-aliases/AuthSchemeType.md)

***

### seed?

> `optional` **seed**: `string` \| `Uint8Array`

***

### storage?

> `optional` **storage**: `"private"` \| `"public"`

***

### type?

> `optional` **type**: `"MutableWallet"` \| `"ImmutableWallet"`

Account type. Defaults to "MutableWallet". Use AccountType enum.
