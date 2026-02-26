[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / ImportAccountInput

# Type Alias: ImportAccountInput

> **ImportAccountInput** = [`AccountRef`](AccountRef.md) \| \{ `file`: [`AccountFile`](../classes/AccountFile.md); \} \| \{ `auth?`: [`AuthSchemeType`](AuthSchemeType.md); `seed`: `Uint8Array`; `type?`: `"MutableWallet"` \| `"ImmutableWallet"`; \}

Discriminated union for account import.

- `AccountRef` (string, AccountId, Account, AccountHeader) — Import a public account by ID (fetches state from the network).
- `{ file: AccountFile }` — Import from a previously exported account file (works for both public and private accounts).
- `{ seed, type?, auth? }` — Reconstruct a **public** account from its init seed. **Does not work for private accounts** — use the account file workflow instead.

## Type Declaration

[`AccountRef`](AccountRef.md)

\{ `file`: [`AccountFile`](../classes/AccountFile.md); \}

### file

> **file**: [`AccountFile`](../classes/AccountFile.md)

\{ `auth?`: [`AuthSchemeType`](AuthSchemeType.md); `seed`: `Uint8Array`; `type?`: `"MutableWallet"` \| `"ImmutableWallet"`; \}

### auth?

> `optional` **auth**: [`AuthSchemeType`](AuthSchemeType.md)

### seed

> **seed**: `Uint8Array`

### type?

> `optional` **type**: `"MutableWallet"` \| `"ImmutableWallet"`

Account type. Defaults to "MutableWallet". Use AccountType enum.
