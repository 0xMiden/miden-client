[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / NoteStorage

# Class: NoteStorage

A container for note storage items.

A note can be associated with up to 1024 storage items. Each item is represented by a single
field element. Thus, note storage can contain up to ~8 KB of data.

All storage items associated with a note can be reduced to a single commitment which is
computed as an RPO256 hash over the storage elements.

## Constructors

### Constructor

> **new NoteStorage**(`felt_array`): `NoteStorage`

Creates note storage from a list of field elements.

#### Parameters

##### felt\_array

[`FeltArray`](FeltArray.md)

#### Returns

`NoteStorage`

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

### items()

> **items**(): [`Felt`](Felt.md)[]

Returns the raw storage items as an array of field elements.

#### Returns

[`Felt`](Felt.md)[]
