[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / AccountStorage

# Class: AccountStorage

## Methods

### commitment()

> **commitment**(): [`Word`](Word.md)

Returns a commitment to this storage.

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

Returns an item from the storage at the specified index.

#### Parameters

##### index

`number`

The slot index in storage.

#### Returns

[`Word`](Word.md)

The stored `Word`, or `undefined` if not found.

#### Remarks

Errors:
- If the index is out of bounds

***

### getMapItem()

> **getMapItem**(`index`, `key`): [`Word`](Word.md)

Retrieves a map item from a map located in storage at the specified index.

#### Parameters

##### index

`number`

The slot index in storage.

##### key

[`Word`](Word.md)

The key used to look up the map item.

#### Returns

[`Word`](Word.md)

The stored `Word`, or `undefined` if not found.

#### Remarks

Errors:
- If the index is out of bounds
- If the indexed storage slot is not a map
