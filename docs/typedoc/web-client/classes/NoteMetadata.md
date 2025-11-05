[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / NoteMetadata

# Class: NoteMetadata

Metadata describing the origin and policy for a note.

## Constructors

### Constructor

> **new NoteMetadata**(`sender`, `note_type`, `note_tag`, `note_execution_hint`, `aux?`): `NoteMetadata`

Creates note metadata from sender, type, tag, execution hint, and optional auxiliary data.

#### Parameters

##### sender

[`AccountId`](AccountId.md)

##### note\_type

[`NoteType`](../enumerations/NoteType.md)

##### note\_tag

[`NoteTag`](NoteTag.md)

##### note\_execution\_hint

[`NoteExecutionHint`](NoteExecutionHint.md)

##### aux?

[`Felt`](Felt.md)

#### Returns

`NoteMetadata`

## Methods

### free()

> **free**(): `void`

#### Returns

`void`

***

### noteType()

> **noteType**(): [`NoteType`](../enumerations/NoteType.md)

Returns the note type (private, encrypted, public).

#### Returns

[`NoteType`](../enumerations/NoteType.md)

***

### sender()

> **sender**(): [`AccountId`](AccountId.md)

Returns the sender account identifier.

#### Returns

[`AccountId`](AccountId.md)

***

### tag()

> **tag**(): [`NoteTag`](NoteTag.md)

Returns the tag describing the note use case.

#### Returns

[`NoteTag`](NoteTag.md)
