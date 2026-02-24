[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / JsAccountUpdate

# Class: JsAccountUpdate

Represents an update to a single account's state.

## Properties

### accountCommitment

> **accountCommitment**: `string`

The cryptographic commitment representing this account's current state.

***

### accountId

> **accountId**: `string`

ID for this account.

***

### assets

> **assets**: [`JsVaultAsset`](JsVaultAsset.md)[]

The account's asset vault.

***

### codeRoot

> **codeRoot**: `string`

The merkle root of the account's executable code.

***

### committed

> **committed**: `boolean`

Whether this account update has been committed.

***

### nonce

> **nonce**: `string`

The account's transaction nonce as a string.

***

### storageMapEntries

> **storageMapEntries**: [`JsStorageMapEntry`](JsStorageMapEntry.md)[]

Serialized storage map entries for this account.

***

### storageRoot

> **storageRoot**: `string`

The merkle root of the account's storage trie.

***

### storageSlots

> **storageSlots**: [`JsStorageSlot`](JsStorageSlot.md)[]

Serialized storage slot data for this account.

***

### vaultRoot

> **vaultRoot**: `string`

The merkle root of the account's asset vault.

## Accessors

### accountSeed

#### Get Signature

> **get** **accountSeed**(): `Uint8Array`

Optional seed data for the account.

##### Returns

`Uint8Array`

#### Set Signature

> **set** **accountSeed**(`value`): `void`

Optional seed data for the account.

##### Parameters

###### value

`Uint8Array`

##### Returns

`void`

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

### toJSON()

> **toJSON**(): `Object`

* Return copy of self without private attributes.

#### Returns

`Object`

***

### toString()

> **toString**(): `string`

Return stringified version of self.

#### Returns

`string`
