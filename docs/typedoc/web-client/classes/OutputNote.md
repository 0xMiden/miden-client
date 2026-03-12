[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / OutputNote

# Class: OutputNote

Representation of a note produced by a transaction (full or partial).

## Methods

### \[dispose\]()

> **\[dispose\]**(): `void`

#### Returns

`void`

***

### assets()

> **assets**(): `NoteAssets`

Returns the assets if they are present.

#### Returns

`NoteAssets`

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

> **metadata**(): `NoteMetadata`

Returns the metadata that accompanies this output.

#### Returns

`NoteMetadata`

***

### recipientDigest()

> **recipientDigest**(): [`Word`](Word.md)

Returns the recipient digest.

#### Returns

[`Word`](Word.md)

***

### full()

> `static` **full**(`note`): `OutputNote`

Wraps a full note output.

#### Parameters

##### note

[`Note`](Note.md)

#### Returns

`OutputNote`
