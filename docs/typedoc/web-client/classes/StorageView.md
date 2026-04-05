[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / StorageView

# Class: StorageView

Wraps WASM AccountStorage with a developer-friendly API.

`getItem()` returns a `StorageResult` that works intuitively for both
Value and StorageMap slots. The raw WASM AccountStorage is accessible
via `.raw`.

Installed on `Account.prototype.storage()` at WASM load time.

## Constructors

### Constructor

> **new StorageView**(): `StorageView`

#### Returns

`StorageView`

## Accessors

### raw

#### Get Signature

> **get** **raw**(): [`AccountStorage`](AccountStorage.md)

The raw WASM AccountStorage.

##### Returns

[`AccountStorage`](AccountStorage.md)

## Methods

### commitment()

> **commitment**(): [`Word`](Word.md)

Returns the commitment to the full account storage.

#### Returns

[`Word`](Word.md)

***

### getCommitment()

> **getCommitment**(`slotName`): [`Word`](Word.md)

Returns the commitment root of a storage slot.
For Value slots: the stored Word. For StorageMap slots: the Merkle root hash.
Useful for proofs, state comparison, and syncing.

#### Parameters

##### slotName

`string`

#### Returns

[`Word`](Word.md)

***

### getItem()

> **getItem**(`slotName`): [`StorageResult`](StorageResult.md)

Smart read: returns a `StorageResult` for the given slot.
For Value slots: wraps the stored Word.
For StorageMap slots: wraps the first entry's value, with all entries in `.entries`.

#### Parameters

##### slotName

`string`

#### Returns

[`StorageResult`](StorageResult.md)

***

### getMapEntries()

> **getMapEntries**(`slotName`): `object`[]

Get all key-value pairs from a StorageMap slot.

#### Parameters

##### slotName

`string`

#### Returns

`object`[]

***

### getMapItem()

> **getMapItem**(`slotName`, `key`): [`Word`](Word.md)

Returns the value for a key in a StorageMap slot.

#### Parameters

##### slotName

`string`

##### key

[`Word`](Word.md)

#### Returns

[`Word`](Word.md)

***

### getNumber()

> **getNumber**(`slotName`): `number`

Convenience: first felt as a JavaScript number.
Works for both slot types.

#### Parameters

##### slotName

`string`

#### Returns

`number`

***

### getSlotNames()

> **getSlotNames**(): `string`[]

Returns the names of all storage slots.

#### Returns

`string`[]
