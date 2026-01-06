[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / StorageSlot

# Class: StorageSlot

A single storage slot value or map for an account component.

## Methods

### \[dispose\]()

> **\[dispose\]**(): `void`

#### Returns

`void`

***

### free()

> **free**(): `void`

#### Returns

`void`

***

### emptyValue()

> `static` **emptyValue**(`name`): `StorageSlot`

Returns an empty value slot (zeroed).

#### Parameters

##### name

`string`

#### Returns

`StorageSlot`

***

### fromValue()

> `static` **fromValue**(`name`, `value`): `StorageSlot`

Creates a storage slot holding a single value.

#### Parameters

##### name

`string`

##### value

[`Word`](Word.md)

#### Returns

`StorageSlot`

***

### map()

> `static` **map**(`name`, `storage_map`): `StorageSlot`

Creates a storage slot backed by a map.

#### Parameters

##### name

`string`

##### storage\_map

[`StorageMap`](StorageMap.md)

#### Returns

`StorageSlot`
