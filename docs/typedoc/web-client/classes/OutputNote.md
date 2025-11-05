[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / OutputNote

# Class: OutputNote

Represents a note produced by executing a transaction.

## Methods

### assets()

> **assets**(): [`NoteAssets`](NoteAssets.md)

Returns the note assets if they are available.

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

Returns the note identifier.

#### Returns

[`NoteId`](NoteId.md)

***

### intoFull()

> **intoFull**(): [`Note`](Note.md)

Consumes the wrapper and returns the full note if available.

#### Returns

[`Note`](Note.md)

***

### metadata()

> **metadata**(): [`NoteMetadata`](NoteMetadata.md)

Returns the note metadata.

#### Returns

[`NoteMetadata`](NoteMetadata.md)

***

### recipientDigest()

> **recipientDigest**(): [`Word`](Word.md)

Returns the recipient digest if known.

#### Returns

[`Word`](Word.md)

***

### shrink()

> **shrink**(): `OutputNote`

Shrinks the note to the minimal representation containing the same information.

#### Returns

`OutputNote`

***

### full()

> `static` **full**(`note`): `OutputNote`

Wraps a full note payload.

#### Parameters

##### note

[`Note`](Note.md)

#### Returns

`OutputNote`

***

### header()

> `static` **header**(`note_header`): `OutputNote`

Wraps only the note header.

#### Parameters

##### note\_header

[`NoteHeader`](NoteHeader.md)

#### Returns

`OutputNote`

***

### partial()

> `static` **partial**(`partial_note`): `OutputNote`

Wraps a partial note payload (header plus metadata).

#### Parameters

##### partial\_note

[`PartialNote`](PartialNote.md)

#### Returns

`OutputNote`
