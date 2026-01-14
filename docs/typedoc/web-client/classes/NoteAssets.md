[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / NoteAssets

# Class: NoteAssets

An asset container for a note.

A note must contain at least 1 asset and can contain up to 256 assets. No duplicates are
allowed, but the order of assets is unspecified.

All the assets in a note can be reduced to a single commitment which is computed by sequentially
hashing the assets. Note that the same list of assets can result in two different commitments if
the asset ordering is different.

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
