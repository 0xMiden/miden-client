[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / SlotAndKeys

# Class: SlotAndKeys

## Constructors

### Constructor

> **new SlotAndKeys**(`storage_slot_index`, `storage_map_keys`): `SlotAndKeys`

Creates a new slot-and-keys entry.

#### Parameters

##### storage\_slot\_index

`number`

##### storage\_map\_keys

[`Word`](Word.md)[]

#### Returns

`SlotAndKeys`

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

### storage\_map\_keys()

> **storage\_map\_keys**(): [`Word`](Word.md)[]

Returns the storage map keys required for this slot.

#### Returns

[`Word`](Word.md)[]

***

### storage\_slot\_index()

> **storage\_slot\_index**(): `number`

Returns the slot index.

#### Returns

`number`
