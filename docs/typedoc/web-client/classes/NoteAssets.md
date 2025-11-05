[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / NoteAssets

# Class: NoteAssets

Collection of assets locked within a note.

## Constructors

### Constructor

> **new NoteAssets**(`assets_array?`): `NoteAssets`

Creates a new note asset list from optional assets.

#### Parameters

##### assets\_array?

[`FungibleAsset`](FungibleAsset.md)[]

#### Returns

`NoteAssets`

## Methods

### free()

> **free**(): `void`

#### Returns

`void`

***

### fungibleAssets()

> **fungibleAssets**(): [`FungibleAsset`](FungibleAsset.md)[]

Returns the fungible assets contained in the note.

#### Returns

[`FungibleAsset`](FungibleAsset.md)[]

***

### push()

> **push**(`asset`): `void`

Adds a fungible asset to the note.

#### Parameters

##### asset

[`FungibleAsset`](FungibleAsset.md)

#### Returns

`void`
