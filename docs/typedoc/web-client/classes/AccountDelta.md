[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / AccountDelta

# Class: AccountDelta

Changes applied to an account's nonce, storage, and vault.

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

### id()

> **id**(): [`AccountId`](AccountId.md)

Returns the affected account ID.

#### Returns

[`AccountId`](AccountId.md)

***

### isEmpty()

> **isEmpty**(): `boolean`

Returns true if there are no changes.

#### Returns

`boolean`

***

### nonceDelta()

> **nonceDelta**(): [`Felt`](Felt.md)

Returns the nonce change.

#### Returns

[`Felt`](Felt.md)

***

### serialize()

> **serialize**(): `Uint8Array`

Serializes the account delta into bytes.

#### Returns

`Uint8Array`

***

### storage()

> **storage**(): [`AccountStorageDelta`](AccountStorageDelta.md)

Returns the storage delta.

#### Returns

[`AccountStorageDelta`](AccountStorageDelta.md)

***

### vault()

> **vault**(): [`AccountVaultDelta`](AccountVaultDelta.md)

Returns the vault delta.

#### Returns

[`AccountVaultDelta`](AccountVaultDelta.md)

***

### deserialize()

> `static` **deserialize**(`bytes`): `AccountDelta`

Deserializes an account delta from bytes.

#### Parameters

##### bytes

`Uint8Array`

#### Returns

`AccountDelta`
