[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / NoteInclusionProof

# Class: NoteInclusionProof

Proof that a note commitment exists at a specific position in the note tree.

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
