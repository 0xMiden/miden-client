[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / AccountId

# Class: AccountId

Uniquely identifies a specific account.

A Miden account ID is a 120-bit value derived from the commitments to account code and storage,
and a random user-provided seed.

## Methods

### \[dispose\]()

> **\[dispose\]**(): `void`

#### Returns

`void`

***

### free()

> **free**(): `void`

#### Returns

`void`

***

### isFaucet()

> **isFaucet**(): `boolean`

Returns true if the ID refers to a faucet.

#### Returns

`boolean`

***

### isNetwork()

> **isNetwork**(): `boolean`

Returns true if the ID is reserved for network accounts.

#### Returns

`boolean`

***

### isPrivate()

> **isPrivate**(): `boolean`

Returns true if the account uses private storage.

#### Returns

`boolean`

***

### isPublic()

> **isPublic**(): `boolean`

Returns true if the account uses public storage.

#### Returns

`boolean`

***

### isRegularAccount()

> **isRegularAccount**(): `boolean`

Returns true if the ID refers to a regular account.

#### Returns

`boolean`

***

### prefix()

> **prefix**(): [`Felt`](Felt.md)

Returns the prefix field element storing metadata about version, type, and storage mode.

#### Returns

[`Felt`](Felt.md)

***

### suffix()

> **suffix**(): [`Felt`](Felt.md)

Returns the suffix field element derived from the account seed.

#### Returns

[`Felt`](Felt.md)

***

### toBech32()

> **toBech32**(`network_id`, `account_interface`): `string`

Will turn the Account ID into its bech32 string representation.

#### Parameters

##### network\_id

`NetworkId`

##### account\_interface

`AccountInterface`

#### Returns

`string`

***

### toString()

> **toString**(): `string`

Returns the canonical hex representation of the account ID.

#### Returns

`string`

***

### fromBech32()

> `static` **fromBech32**(`bech_32_encoded_id`): `AccountId`

Given a bech32 encoded string, return the matching Account ID for it.

#### Parameters

##### bech\_32\_encoded\_id

`string`

#### Returns

`AccountId`

***

### fromHex()

> `static` **fromHex**(`hex`): `AccountId`

Builds an account ID from its hex string representation.

Returns an error if the provided string is not a valid hex-encoded account ID.

#### Parameters

##### hex

`string`

#### Returns

`AccountId`
