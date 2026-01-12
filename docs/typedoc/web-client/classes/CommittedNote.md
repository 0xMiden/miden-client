[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / CommittedNote

# Class: CommittedNote

Represents a note committed on chain, as returned by `syncNotes`.

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

### inclusionPath()

> **inclusionPath**(): [`SparseMerklePath`](SparseMerklePath.md)

Returns the inclusion path for the note.

#### Returns

[`SparseMerklePath`](SparseMerklePath.md)

***

### metadata()

> **metadata**(): [`NoteMetadata`](NoteMetadata.md)

Returns the note metadata.

#### Returns

[`NoteMetadata`](NoteMetadata.md)

***

### noteId()

> **noteId**(): [`NoteId`](NoteId.md)

Returns the note ID.

#### Returns

[`NoteId`](NoteId.md)

***

### noteIndex()

> **noteIndex**(): `number`

Returns the note index in the block's note tree.

#### Returns

`number`
