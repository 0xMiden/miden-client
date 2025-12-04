[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / PartialNote

# Class: PartialNote

Partial information about a note.

Partial note consists of [`NoteMetadata`], [`NoteAssets`], and a recipient digest (see
[`super::NoteRecipient`]). However, it does not contain detailed recipient info, including note
script, note inputs, and note's serial number. This means that a partial note is sufficient to
compute note ID and note header, but not sufficient to compute note nullifier, and generally
does not have enough info to execute the note.

## Methods

### \[dispose\]()

> **\[dispose\]**(): `void`

#### Returns

`void`

***

### assets()

> **assets**(): [`NoteAssets`](NoteAssets.md)

Returns the assets locked in the note.

#### Returns

[`NoteAssets`](NoteAssets.md)

***

### free()

> **free**(): `void`

#### Returns

`void`

***

### id()

> **id**(): [`NoteId`](NoteId.md)

Returns the identifier of the partial note.

#### Returns

[`NoteId`](NoteId.md)

***

### metadata()

> **metadata**(): [`NoteMetadata`](NoteMetadata.md)

Returns the metadata attached to the note.

#### Returns

[`NoteMetadata`](NoteMetadata.md)

***

### recipientDigest()

> **recipientDigest**(): [`Word`](Word.md)

Returns the digest of the recipient information.

#### Returns

[`Word`](Word.md)
