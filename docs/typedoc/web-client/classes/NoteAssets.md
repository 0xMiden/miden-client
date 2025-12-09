[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / NoteAssets

# Class: NoteAssets

## Constructors

### Constructor

> **new NoteAssets**(`assets_array?`): `NoteAssets`

Creates a new asset list for a note.

#### Parameters

##### assets\_array?

[`FungibleAsset`](FungibleAsset.md)[]

#### Returns

`NoteAssets`

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

### fungibleAssets()

> **fungibleAssets**(): [`FungibleAsset`](FungibleAsset.md)[]

Returns all fungible assets contained in the note.

#### Returns

[`FungibleAsset`](FungibleAsset.md)[]

***

### push()

> **push**(`asset`): `void`

Adds a fungible asset to the collection.

#### Parameters

##### asset

[`FungibleAsset`](FungibleAsset.md)

#### Returns

`void`
