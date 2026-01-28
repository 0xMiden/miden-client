[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / NoteInclusionProof

# Class: NoteInclusionProof

Contains the data required to prove inclusion of a note in the canonical chain.

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

### location()

> **location**(): [`NoteLocation`](NoteLocation.md)

Returns the location of the note within the tree.

#### Returns

[`NoteLocation`](NoteLocation.md)

***

### notePath()

> **notePath**(): [`MerklePath`](MerklePath.md)

Returns the Merkle authentication path for the note.

#### Returns

[`MerklePath`](MerklePath.md)
