[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / AccountStorage

# Class: AccountStorage

Account storage is composed of a variable number of index-addressable storage slots up to 255
slots in total.

Each slot has a type which defines its size and structure. Currently, the following types are
supported:
- `StorageSlot::Value`: contains a single Word of data (i.e., 32 bytes).
- `StorageSlot::Map`: contains a `StorageMap` which is a key-value map where both keys and
  values are Words. The value of a storage slot containing a map is the commitment to the
  underlying map.

## Methods

### \[dispose\]()

> **\[dispose\]**(): `void`

#### Returns

`void`

***

### commitment()

> **commitment**(): [`Word`](Word.md)

Returns the commitment to the full account storage.

#### Returns

[`Word`](Word.md)

***

### free()

> **free**(): `void`

#### Returns

`void`

***

### getItem()

> **getItem**(`index`): [`Word`](Word.md)

Returns the value stored at the given slot index, if any.

#### Parameters

##### index

`number`

#### Returns

[`Word`](Word.md)

***

### getMapEntries()

> **getMapEntries**(`slot_name`): [`JsStorageMapEntry`](JsStorageMapEntry.md)[]

Get all key-value pairs from the map slot identified by `slot_name`.
Returns `undefined` if the slot isn't a map or doesn't exist.
Returns `[]` if the map exists but is empty.

#### Parameters

##### slot\_name

`string`

#### Returns

[`JsStorageMapEntry`](JsStorageMapEntry.md)[]

***

### getMapItem()

> **getMapItem**(`slot_name`, `key`): [`Word`](Word.md)

Returns the value for a key in the map stored at the given slot, if any.

#### Parameters

##### slot\_name

`string`

##### key

[`Word`](Word.md)

#### Returns

[`Word`](Word.md)
