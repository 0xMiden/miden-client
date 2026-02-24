[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / ImportAccountInput

# Type Alias: ImportAccountInput

> **ImportAccountInput** = `string` \| \{ `file`: [`AccountFile`](../classes/AccountFile.md); \} \| \{ `type?`: `"MutableWallet"` \| `"ImmutableWallet"`; `auth?`: [`AuthSchemeType`](AuthSchemeType.md); `seed`: `Uint8Array`; \}

Discriminated union for account import.

## Type Declaration

`string`

\{ `file`: [`AccountFile`](../classes/AccountFile.md); \}

### file

> **file**: [`AccountFile`](../classes/AccountFile.md)

\{ `type?`: `"MutableWallet"` \| `"ImmutableWallet"`; `auth?`: [`AuthSchemeType`](AuthSchemeType.md); `seed`: `Uint8Array`; \}

### type?

> `optional` **type**: `"MutableWallet"` \| `"ImmutableWallet"`

Account type. Defaults to "MutableWallet". Use AccountType enum.

### auth?

> `optional` **auth**: [`AuthSchemeType`](AuthSchemeType.md)

### seed

> **seed**: `Uint8Array`
