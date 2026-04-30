[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / AccountType

# Variable: AccountType

> **AccountType**: `object`

Account type constants with numeric values matching the WASM `AccountType` enum.
Includes SDK-friendly aliases (e.g. `MutableWallet`) that map to the same
numeric values. These values work with both `accounts.create()` and the
low-level `AccountBuilder.accountType()`.

## Type Declaration

### FungibleFaucet

> `readonly` **FungibleFaucet**: `0`

### ImmutableContract

> `readonly` **ImmutableContract**: `2`

### ImmutableWallet

> `readonly` **ImmutableWallet**: `2`

### MutableContract

> `readonly` **MutableContract**: `3`

### MutableWallet

> `readonly` **MutableWallet**: `3`

### NonFungibleFaucet

> `readonly` **NonFungibleFaucet**: `1`

### RegularAccountImmutableCode

> `readonly` **RegularAccountImmutableCode**: `2`

### RegularAccountUpdatableCode

> `readonly` **RegularAccountUpdatableCode**: `3`
