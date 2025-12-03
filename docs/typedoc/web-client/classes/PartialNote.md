[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / PartialNote

# Class: PartialNote

Note variant exposing assets and metadata but hiding full recipient details.

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
