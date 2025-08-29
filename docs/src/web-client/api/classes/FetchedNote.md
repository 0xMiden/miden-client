[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / FetchedNote

# Class: FetchedNote

Represents a note fetched from a Miden node via RPC.

## Constructors

### Constructor

> **new FetchedNote**(`note_id`, `metadata`, `input_note`?): `FetchedNote`

Create a note with an optional `InputNote`.

#### Parameters

##### note\_id

[`NoteId`](NoteId.md)

##### metadata

[`NoteMetadata`](NoteMetadata.md)

##### input\_note?

[`InputNote`](InputNote.md)

#### Returns

`FetchedNote`

## Properties

### inputNote

> `readonly` **inputNote**: [`InputNote`](InputNote.md)

The full [`InputNote`] with inclusion proof.

For public notes, it contains the complete note data and inclusion proof.
For private notes, it will be ``None`.

***

### metadata

> `readonly` **metadata**: [`NoteMetadata`](NoteMetadata.md)

The note's metadata, including sender, tag, and other properties.
Available for both private and public notes.

***

### noteId

> `readonly` **noteId**: [`NoteId`](NoteId.md)

The unique identifier of the note.

***

### noteType

> `readonly` **noteType**: [`NoteType`](../enumerations/NoteType.md)

## Methods

### free()

> **free**(): `void`

#### Returns

`void`
