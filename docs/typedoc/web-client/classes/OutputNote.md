[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / OutputNote

# Class: OutputNote

## Methods

### \[dispose\]()

> **\[dispose\]**(): `void`

#### Returns

`void`

***

### assets()

> **assets**(): [`NoteAssets`](NoteAssets.md)

Returns the assets if they are present.

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

Returns the note ID for this output.

#### Returns

[`NoteId`](NoteId.md)

***

### intoFull()

> **intoFull**(): [`Note`](Note.md)

Converts into a full note if the data is present.

#### Returns

[`Note`](Note.md)

***

### metadata()

> **metadata**(): [`NoteMetadata`](NoteMetadata.md)

Returns the metadata that accompanies this output.

#### Returns

[`NoteMetadata`](NoteMetadata.md)

***

### recipientDigest()

> **recipientDigest**(): [`Word`](Word.md)

Returns the recipient digest if the recipient is known.

#### Returns

[`Word`](Word.md)

***

### shrink()

> **shrink**(): `OutputNote`

Returns a more compact representation if possible (e.g. dropping details).

#### Returns

`OutputNote`

***

### full()

> `static` **full**(`note`): `OutputNote`

Wraps a full note output.

#### Parameters

##### note

[`Note`](Note.md)

#### Returns

`OutputNote`

***

### header()

> `static` **header**(`note_header`): `OutputNote`

Wraps only the header of a note.

#### Parameters

##### note\_header

[`NoteHeader`](NoteHeader.md)

#### Returns

`OutputNote`

***

### partial()

> `static` **partial**(`partial_note`): `OutputNote`

Wraps a partial note containing assets and recipient only.

#### Parameters

##### partial\_note

[`PartialNote`](PartialNote.md)

#### Returns

`OutputNote`
