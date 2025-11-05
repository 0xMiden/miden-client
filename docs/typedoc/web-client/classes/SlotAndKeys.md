[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / SlotAndKeys

# Class: SlotAndKeys

Helper structure representing a storage slot index with the keys to retain.

## Constructors

### Constructor

> **new SlotAndKeys**(`storage_slot_index`, `storage_map_keys`): `SlotAndKeys`

Creates a new [`SlotAndKeys`] entry.

#### Parameters

##### storage\_slot\_index

`number`

##### storage\_map\_keys

[`Word`](Word.md)[]

#### Returns

`SlotAndKeys`

## Methods

### free()

> **free**(): `void`

#### Returns

`void`

***

### storage\_map\_keys()

> **storage\_map\_keys**(): [`Word`](Word.md)[]

Returns the map keys that must be available for this slot.

#### Returns

[`Word`](Word.md)[]

***

### storage\_slot\_index()

> **storage\_slot\_index**(): `number`

Returns the index of the storage slot.

#### Returns

`number`
