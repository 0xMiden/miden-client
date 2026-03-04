[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / NoteMetadata

# Class: NoteMetadata

Metadata associated with a note.

This metadata includes the sender, note type, tag, and an optional attachment.
Attachments provide additional context about how notes should be processed.

## Constructors

### Constructor

> **new NoteMetadata**(`sender`, `note_type`, `note_tag`): `NoteMetadata`

Creates metadata for a note.

#### Parameters

##### sender

[`AccountId`](AccountId.md)

##### note\_type

[`NoteType`](../enumerations/NoteType.md)

##### note\_tag

[`NoteTag`](NoteTag.md)

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

***

### withAttachment()

> **withAttachment**(`attachment`): `NoteMetadata`

Adds an attachment to this metadata and returns the updated metadata.

Attachments provide additional context about how notes should be processed.
For example, a `NetworkAccountTarget` attachment indicates that the note
should be consumed by a specific network account.

#### Parameters

##### attachment

[`NoteAttachment`](NoteAttachment.md)

#### Returns

`NoteMetadata`
