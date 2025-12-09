[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / NoteMetadata

# Class: NoteMetadata

## Constructors

### Constructor

> **new NoteMetadata**(`sender`, `note_type`, `note_tag`, `note_execution_hint`, `aux?`): `NoteMetadata`

Creates metadata for a note.

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

### noteType()

> **noteType**(): [`NoteType`](../enumerations/NoteType.md)

Returns whether the note is private, encrypted, or public.

#### Returns

[`NoteType`](../enumerations/NoteType.md)

***

### sender()

> **sender**(): [`AccountId`](AccountId.md)

Returns the account that created the note.

#### Returns

[`AccountId`](AccountId.md)

***

### tag()

> **tag**(): [`NoteTag`](NoteTag.md)

Returns the tag associated with the note.

#### Returns

[`NoteTag`](NoteTag.md)
