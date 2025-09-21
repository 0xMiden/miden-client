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

Returns all entries from the storage map at the given index.
Returns an empty array if the slot is not a map or if the index is out of bounds.

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
