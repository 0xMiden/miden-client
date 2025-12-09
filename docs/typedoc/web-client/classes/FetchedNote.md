[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / FetchedNote

# Class: FetchedNote

Represents a note fetched from a Miden node via RPC.

## Constructors

### Constructor

> **new FetchedNote**(`note_id`, `metadata`, `note`, `inclusion_proof`): `FetchedNote`

Create a `FetchedNote` with an optional [`Note`].

#### Parameters

##### note\_id

[`NoteId`](NoteId.md)

##### metadata

[`NoteMetadata`](NoteMetadata.md)

##### note

[`Note`](Note.md)

##### inclusion\_proof

[`NoteInclusionProof`](NoteInclusionProof.md)

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
For private notes, it will be `None`.

***

### noteId

> `readonly` **noteId**: [`NoteId`](NoteId.md)

The unique identifier of the note.

***

### noteType

> `readonly` **noteType**: [`NoteType`](../enumerations/NoteType.md)

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
