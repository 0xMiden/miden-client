[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / AccountStorage

# Class: AccountStorage

WASM-facing view over account storage slots.

## Methods

### commitment()

> **commitment**(): [`Word`](Word.md)

Returns the storage commitment of the account.

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

Returns the value stored at the given index if it is a value slot.

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

Returns the value stored under `key` for the map slot at `index`.

#### Parameters

##### index

`number`

##### key

[`Word`](Word.md)

#### Returns

[`Word`](Word.md)
