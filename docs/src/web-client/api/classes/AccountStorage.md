[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / AccountStorage

# Class: AccountStorage

## Methods

### commitment()

> **commitment**(): [`Word`](Word.md)

#### Returns

[`Word`](Word.md)

***

### forEachMapEntry()

> **forEachMapEntry**(`index`, `callback`): `void`

Stream all key-value pairs from the map slot at `index` via a callback function.
This is a memory-efficient alternative to `getMapEntries` for large maps.

The callback receives a `JsStorageMapEntry` object with `root`, `key`, and `value` fields.
Entries are processed one at a time without allocating an intermediate vector.

Returns an error if:
- The slot at `index` is not a map
- `index` is out of bounds (0-255)
- The callback throws an error during execution

#### Parameters

##### index

`number`

##### callback

`Function`

#### Returns

`void`

***

### free()

> **free**(): `void`

#### Returns

`void`

***

### getItem()

> **getItem**(`index`): [`Word`](Word.md)

#### Parameters

##### index

`number`

#### Returns

[`Word`](Word.md)

***

### getMapEntries()

> **getMapEntries**(`index`): [`JsStorageMapEntry`](JsStorageMapEntry.md)[]

Get all key-value pairs from the map slot at `index`.
Returns `undefined` if the slot isn't a map or `index` is out of bounds (0-255).
Returns `[]` if the map exists but is empty.

WARNING: This method allocates the entire map into memory.
For large maps, use `forEachMapEntry` instead for better memory efficiency.

#### Parameters

##### index

`number`

#### Returns

[`JsStorageMapEntry`](JsStorageMapEntry.md)[]

***

### getMapItem()

> **getMapItem**(`index`, `key`): [`Word`](Word.md)

#### Parameters

##### index

`number`

##### key

[`Word`](Word.md)

#### Returns

[`Word`](Word.md)
