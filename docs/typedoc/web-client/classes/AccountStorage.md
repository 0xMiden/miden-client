[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / AccountStorage

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

> **getItem**(`slot_name`): [`Word`](Word.md)

Returns the value stored at the given slot name.
For Value slots: returns the stored Word directly.
For Map slots: returns the first entry's value (NOT the commitment hash).

#### Parameters

##### slot\_name

`string`

#### Returns

[`Word`](Word.md)

***

### getMapEntries()

> **getMapEntries**(`slot_name`): `JsStorageMapEntry`[]

Get all key-value pairs from the map slot identified by `slot_name`.
Returns `undefined` if the slot isn't a map or doesn't exist.
Returns `[]` if the map exists but is empty.

#### Parameters

##### slot\_name

`string`

#### Returns

`JsStorageMapEntry`[]

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

***

### getNumber()

> **getNumber**(`slot_name`): `number`

Returns the first felt of a storage slot as a number.
Works for both Value and Map slots (for maps, returns first entry's first felt).

#### Parameters

##### slot\_name

`string`

#### Returns

`number`

***

### getSlotNames()

> **getSlotNames**(): `string`[]

Returns the names of all storage slots on this account.

#### Returns

`string`[]

***

### readNumber()

> **readNumber**(`slot_name`, `key?`): `number`

Convenience: read a storage value and return the first felt as a number.
Handles both Value and Map slots.

#### Parameters

##### slot\_name

`string`

##### key?

[`Word`](Word.md)

#### Returns

`number`

***

### readValue()

> **readValue**(`slot_name`, `key?`): [`Word`](Word.md)

Smart read: returns the actual stored value regardless of slot type.
- For Value slots: returns the stored Word directly.
- For Map slots: returns the value at the given key, or the first entry's value if no key.

#### Parameters

##### slot\_name

`string`

##### key?

[`Word`](Word.md)

#### Returns

[`Word`](Word.md)
