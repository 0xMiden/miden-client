[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / StorageSlot

# Class: StorageSlot

Represents a single storage slot within an account (value or map).

## Methods

### free()

> **free**(): `void`

#### Returns

`void`

***

### emptyValue()

> `static` **emptyValue**(): `StorageSlot`

Returns an empty value slot.

#### Returns

`StorageSlot`

***

### fromValue()

> `static` **fromValue**(`value`): `StorageSlot`

Creates a slot holding a value word.

#### Parameters

##### value

[`Word`](Word.md)

#### Returns

`StorageSlot`

***

### map()

> `static` **map**(`storage_map`): `StorageSlot`

Creates a slot containing a storage map.

#### Parameters

##### storage\_map

[`StorageMap`](StorageMap.md)

#### Returns

`StorageSlot`
