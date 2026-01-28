[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / FetchedNote

# Class: FetchedNote

Wrapper for a note fetched over RPC.

It contains the note header and inclusion proof. The note details are only present for
public notes.

## Constructors

### Constructor

> **new FetchedNote**(`note_id`, `metadata`, `inclusion_proof`, `note?`): `FetchedNote`

Create a `FetchedNote` with an optional [`Note`].

#### Parameters

##### note\_id

[`NoteId`](NoteId.md)

##### metadata

[`NoteMetadata`](NoteMetadata.md)

##### inclusion\_proof

[`NoteInclusionProof`](NoteInclusionProof.md)

##### note?

[`Note`](Note.md)

#### Returns

`FetchedNote`

## Properties

### header

> `readonly` **header**: [`NoteHeader`](NoteHeader.md)

The note's header, containing the ID and metadata.

***

### inclusionProof

> `readonly` **inclusionProof**: [`NoteInclusionProof`](NoteInclusionProof.md)

The note's inclusion proof.

Contains the data required to prove inclusion of the note in the canonical chain.

***

### metadata

> `readonly` **metadata**: [`NoteMetadata`](NoteMetadata.md)

The note's metadata, including sender, tag, and other properties.
Available for both private and public notes.

***

### note

> `readonly` **note**: [`Note`](Note.md)

The full [`Note`] data.

For public notes, it contains the complete note data.
For private notes, it will be undefined.

***

### noteId

> `readonly` **noteId**: [`NoteId`](NoteId.md)

The unique identifier of the note.

***

### noteType

> `readonly` **noteType**: [`NoteType`](../enumerations/NoteType.md)

Returns whether the note is private, encrypted, or public.

## Methods

### \[dispose\]()

> **\[dispose\]**(): `void`

#### Returns

`void`

***

### asInputNote()

> **asInputNote**(): [`InputNote`](InputNote.md)

Returns an [`InputNote`] when the fetched note is public.

Returns `undefined` when the note body is missing (e.g. private notes); in that case build
an `InputNote` manually using the inclusion proof and note data obtained elsewhere.

#### Returns

[`InputNote`](InputNote.md)

***

### free()

> **free**(): `void`

#### Returns

`void`
