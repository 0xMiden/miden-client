[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / AccountDelta

# Class: AccountDelta

WASM wrapper around [`miden_client::account::AccountDelta`].

Captures the changes applied to an account after executing a transaction.

## Methods

### free()

> **free**(): `void`

#### Returns

`void`

***

### id()

> **id**(): [`AccountId`](AccountId.md)

Returns the identifier of the account the delta belongs to.

#### Returns

[`AccountId`](AccountId.md)

***

### isEmpty()

> **isEmpty**(): `boolean`

Returns `true` if the delta does not change vault, storage, or nonce.

#### Returns

`boolean`

***

### nonceDelta()

> **nonceDelta**(): [`Felt`](Felt.md)

Returns the nonce delta applied to the account.

#### Returns

[`Felt`](Felt.md)

***

### serialize()

> **serialize**(): `Uint8Array`

Serializes the delta into raw bytes.

#### Returns

`Uint8Array`

***

### storage()

> **storage**(): [`AccountStorageDelta`](AccountStorageDelta.md)

Returns the storage-specific changes contained in this delta.

#### Returns

[`AccountStorageDelta`](AccountStorageDelta.md)

***

### vault()

> **vault**(): [`AccountVaultDelta`](AccountVaultDelta.md)

Returns the vault-specific changes contained in this delta.

#### Returns

[`AccountVaultDelta`](AccountVaultDelta.md)

***

### deserialize()

> `static` **deserialize**(`bytes`): `AccountDelta`

Deserializes an account delta from bytes produced by [`serialize`].

#### Parameters

##### bytes

`Uint8Array`

#### Returns

`AccountDelta`

#### Throws

Throws if the bytes cannot be parsed.
