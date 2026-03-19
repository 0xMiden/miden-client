[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / AccountsResource

# Interface: AccountsResource

## Methods

### addAddress()

> **addAddress**(`accountId`, `address`): `Promise`\<`void`\>

Associate a Bech32 address with an account.

#### Parameters

##### accountId

[`AccountRef`](../type-aliases/AccountRef.md)

The account to add the address to.

##### address

`string`

The Bech32 address string.

#### Returns

`Promise`\<`void`\>

***

### create()

> **create**(`options?`): `Promise`\<[`Account`](../classes/Account.md)\>

Create a new wallet, faucet, or contract account. Defaults to a mutable
wallet if no options are provided.

#### Parameters

##### options?

[`CreateAccountOptions`](../type-aliases/CreateAccountOptions.md)

Account creation options discriminated by `type` field.

#### Returns

`Promise`\<[`Account`](../classes/Account.md)\>

***

### export()

> **export**(`accountId`, `options?`): `Promise`\<[`AccountFile`](../classes/AccountFile.md)\>

Export an account to an [AccountFile](../classes/AccountFile.md) for backup or transfer.

#### Parameters

##### accountId

[`AccountRef`](../type-aliases/AccountRef.md)

The account to export.

##### options?

[`ExportAccountOptions`](ExportAccountOptions.md)

Export options (reserved for future use).

#### Returns

`Promise`\<[`AccountFile`](../classes/AccountFile.md)\>

***

### get()

> **get**(`accountId`): `Promise`\<[`Account`](../classes/Account.md)\>

Retrieve an account by ID. Returns `null` if not found in the local store.

#### Parameters

##### accountId

[`AccountRef`](../type-aliases/AccountRef.md)

The account to retrieve.

#### Returns

`Promise`\<[`Account`](../classes/Account.md)\>

***

### getBalance()

> **getBalance**(`accountId`, `tokenId`): `Promise`\<`bigint`\>

Get the balance of a specific token for an account.

#### Parameters

##### accountId

[`AccountRef`](../type-aliases/AccountRef.md)

The account to check.

##### tokenId

[`AccountRef`](../type-aliases/AccountRef.md)

The faucet account that identifies the token.

#### Returns

`Promise`\<`bigint`\>

***

### getDetails()

> **getDetails**(`accountId`): `Promise`\<[`AccountDetails`](AccountDetails.md)\>

Retrieve detailed account information including vault, storage, code, and keys.

#### Parameters

##### accountId

[`AccountRef`](../type-aliases/AccountRef.md)

The account to retrieve details for.

#### Returns

`Promise`\<[`AccountDetails`](AccountDetails.md)\>

***

### getOrImport()

> **getOrImport**(`accountId`): `Promise`\<[`Account`](../classes/Account.md)\>

Retrieve an account locally, or import it from the network if not found.

#### Parameters

##### accountId

[`AccountRef`](../type-aliases/AccountRef.md)

The account to retrieve or import.

#### Returns

`Promise`\<[`Account`](../classes/Account.md)\>

***

### import()

> **import**(`input`): `Promise`\<[`Account`](../classes/Account.md)\>

Import an account from the network by ID, from an exported file, or
reconstruct from a seed.

#### Parameters

##### input

[`ImportAccountInput`](../type-aliases/ImportAccountInput.md)

Account reference, file, or seed-based import options.

#### Returns

`Promise`\<[`Account`](../classes/Account.md)\>

***

### insert()

> **insert**(`account`, `overwrite?`): `Promise`\<`void`\>

Insert a pre-built account into the local store. Useful for external signer
integrations that construct accounts via `AccountBuilder` with custom auth commitments.

#### Parameters

##### account

[`Account`](../classes/Account.md)

The account to insert.

##### overwrite?

`boolean`

Whether to overwrite an existing account with the same ID. Defaults to `false`.

#### Returns

`Promise`\<`void`\>

***

### list()

> **list**(): `Promise`\<[`AccountHeader`](../classes/AccountHeader.md)[]\>

List all accounts in the local store.

#### Returns

`Promise`\<[`AccountHeader`](../classes/AccountHeader.md)[]\>

***

### removeAddress()

> **removeAddress**(`accountId`, `address`): `Promise`\<`void`\>

Remove a Bech32 address from an account.

#### Parameters

##### accountId

[`AccountRef`](../type-aliases/AccountRef.md)

The account to remove the address from.

##### address

`string`

The Bech32 address string to remove.

#### Returns

`Promise`\<`void`\>
