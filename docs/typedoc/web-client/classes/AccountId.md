[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / AccountId

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

Will turn the Account ID into its bech32 string representation. To avoid a potential
wrongful encoding, this function will expect only IDs for either mainnet ("mm"),
testnet ("mtst") or devnet ("mdev"). To use a custom bech32 prefix, see
`Self::to_bech_32_custom`.

#### Parameters

##### network\_id

[`NetworkId`](../enumerations/NetworkId.md)

##### account\_interface

[`BasicWallet`](../enumerations/AccountInterface.md#basicwallet)

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

[`BasicWallet`](../enumerations/AccountInterface.md#basicwallet)

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

#### Parameters

##### hex

`string`

#### Returns

`AccountId`
