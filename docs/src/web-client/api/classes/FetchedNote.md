---
title: FetchedNote
draft: true
---

[**@demox-labs/miden-sdk**](../index)

***

[@demox-labs/miden-sdk](../index) / FetchedNote

# Class: FetchedNote

Represents a note fetched from a Miden node via RPC.

## Constructors

### Constructor

> **new FetchedNote**(`note_id`, `metadata`, `input_note?`): `FetchedNote`

Create a note with an optional `InputNote`.

#### Parameters

##### note\_id

[`NoteId`](NoteId)

##### metadata

[`NoteMetadata`](NoteMetadata)

##### input\_note?

[`InputNote`](InputNote)

#### Returns

`FetchedNote`

## Properties

### inputNote

> `readonly` **inputNote**: [`InputNote`](InputNote)

The full [`InputNote`] with inclusion proof.

For public notes, it contains the complete note data and inclusion proof.
For private notes, it will be ``None`.

***

### metadata

> `readonly` **metadata**: [`NoteMetadata`](NoteMetadata)

The note's metadata, including sender, tag, and other properties.
Available for both private and public notes.

***

### noteId

> `readonly` **noteId**: [`NoteId`](NoteId)

The unique identifier of the note.

***

### noteType

> `readonly` **noteType**: [`NoteType`](../enumerations/NoteType)

## Methods

### free()

> **free**(): `void`

#### Returns

`void`
