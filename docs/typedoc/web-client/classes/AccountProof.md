[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / AccountProof

# Class: AccountProof

Proof of existence of a public account's state at a specific block number, as returned by
the node.

Includes the account header, storage slot values, account code, and optionally storage
map entries for the requested storage maps.

## Methods

### \[dispose\]()

> **\[dispose\]**(): `void`

#### Returns

`void`

***

### accountCode()

> **accountCode**(): [`AccountCode`](AccountCode.md)

Returns the account code, if available (public accounts only).

#### Returns

[`AccountCode`](AccountCode.md)

***

### accountCommitment()

> **accountCommitment**(): [`Word`](Word.md)

Returns the account commitment (hash of the full state).

#### Returns

[`Word`](Word.md)

***

### accountHeader()

> **accountHeader**(): [`AccountHeader`](AccountHeader.md)

Returns the account header, if available (public accounts only).

#### Returns

[`AccountHeader`](AccountHeader.md)

***

### accountId()

> **accountId**(): [`AccountId`](AccountId.md)

Returns the account ID.

#### Returns

[`AccountId`](AccountId.md)

***

### blockNum()

> **blockNum**(): `number`

Returns the block number at which this proof was retrieved.

#### Returns

`number`

***

### free()

> **free**(): `void`

#### Returns

`void`

***

### getStorageMapEntries()

> **getStorageMapEntries**(`slot_name`): [`StorageMapEntry`](StorageMapEntry.md)[]

Returns storage map entries for a given slot name, if available.

Returns `undefined` if the account is private, the slot was not requested in the
storage requirements, or the slot is not a map.

Each entry contains a `key` and `value` as `Word` objects.

#### Parameters

##### slot\_name

`string`

#### Returns

[`StorageMapEntry`](StorageMapEntry.md)[]

***

### getStorageMapSlotNames()

> **getStorageMapSlotNames**(): `string`[]

Returns the names of all storage slots that have map details available.

This can be used to discover which storage maps were included in the proof response.
Returns `undefined` if the account is private.

#### Returns

`string`[]

***

### getStorageSlotValue()

> **getStorageSlotValue**(`slot_name`): [`Word`](Word.md)

Returns the value of a storage slot by name, if available.

For `Value` slots, this returns the stored word.
For `Map` slots, this returns the map root commitment.

Returns `undefined` if the account is private or the slot name is not found.

#### Parameters

##### slot\_name

`string`

#### Returns

[`Word`](Word.md)

***

### hasStorageMapTooManyEntries()

> **hasStorageMapTooManyEntries**(`slot_name`): `boolean`

Returns whether a storage map slot had too many entries to return inline.

When this returns `true`, use `RpcClient.syncStorageMaps()` to fetch the full
storage map data.

Returns `undefined` if the slot was not found or the account is private.

#### Parameters

##### slot\_name

`string`

#### Returns

`boolean`

***

### numStorageSlots()

> **numStorageSlots**(): `number`

Returns the number of storage slots, if available (public accounts only).

#### Returns

`number`
