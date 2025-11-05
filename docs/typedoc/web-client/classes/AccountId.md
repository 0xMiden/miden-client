[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / AccountId

# Class: AccountId

Identifier for an account exposed to JavaScript.

Wraps [`miden_client::account::AccountId`] and provides convenience helpers for formatting and
network-aware conversions.

## Methods

### free()

> **free**(): `void`

#### Returns

`void`

***

### isFaucet()

> **isFaucet**(): `boolean`

Returns `true` if the identifier belongs to a faucet account.

#### Returns

`boolean`

***

### isRegularAccount()

> **isRegularAccount**(): `boolean`

Returns `true` if the identifier belongs to a regular account.

#### Returns

`boolean`

***

### prefix()

> **prefix**(): [`Felt`](Felt.md)

Returns the high-word prefix of the account identifier.

#### Returns

[`Felt`](Felt.md)

***

### suffix()

> **suffix**(): [`Felt`](Felt.md)

Returns the low-word suffix of the account identifier.

#### Returns

[`Felt`](Felt.md)

***

### toBech32()

> **toBech32**(`network_id`, `account_interface`): `string`

Will turn the Account ID into its bech32 string representation. To avoid a potential
wrongful encoding, this function will expect only IDs for either mainnet ("mm"),
testnet ("mtst") or devnet ("mdev"). To use a custom bech32 prefix, see
`Self::to_bech_32_custom`.

#### Parameters

##### network\_id

[`NetworkId`](../enumerations/NetworkId.md)

##### account\_interface

[`AccountInterface`](../enumerations/AccountInterface.md)

#### Returns

`string`

***

### toBech32Custom()

> **toBech32Custom**(`custom_network_id`, `account_interface`): `string`

Turn this Account ID into its bech32 string representation. This method accepts a custom
network ID.

#### Parameters

##### custom\_network\_id

`string`

##### account\_interface

[`AccountInterface`](../enumerations/AccountInterface.md)

#### Returns

`string`

***

### toString()

> **toString**(): `string`

Returns the canonical hex representation of this identifier.

#### Returns

`string`

***

### fromHex()

> `static` **fromHex**(`hex`): `AccountId`

Parses an account identifier from a hex string.

#### Parameters

##### hex

`string`

#### Returns

`AccountId`
