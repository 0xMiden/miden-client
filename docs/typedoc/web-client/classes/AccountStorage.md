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

Get all key-value pairs from the map slot at `index`.
Returns `undefined` if the slot isn't a map or `index` is out of bounds (0-255).
Returns `[]` if the map exists but is empty.

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
