[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / NoteFile

# Class: NoteFile

A serialized representation of a note.

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

> **noteType**(): `string`

Returns this `NoteFile`'s types.

#### Returns

`string`

***

### serialize()

> **serialize**(): `Uint8Array`

Turn a notefile into its byte representation.

#### Returns

`Uint8Array`

***

### toJSON()

> **toJSON**(): `Object`

* Return copy of self without private attributes.

#### Returns

`Object`

***

### toString()

> **toString**(): `string`

Return stringified version of self.

#### Returns

`string`

***

### deserialize()

> `static` **deserialize**(`bytes`): `NoteFile`

Given a valid byte representation of a `NoteFile`,
return it as a struct.

#### Parameters

##### bytes

`Uint8Array`

#### Returns

`NoteFile`

***

### fromInputNote()

> `static` **fromInputNote**(`note`): `NoteFile`

Creates a `NoteFile` from an input note, preserving proof when available.

#### Parameters

##### note

[`InputNote`](InputNote.md)

#### Returns

`NoteFile`

***

### fromNoteDetails()

> `static` **fromNoteDetails**(`note_details`): `NoteFile`

Creates a `NoteFile` from note details.

#### Parameters

##### note\_details

[`NoteDetails`](NoteDetails.md)

#### Returns

`NoteFile`

***

### fromNoteId()

> `static` **fromNoteId**(`note_details`): `NoteFile`

Creates a `NoteFile` from a note ID.

#### Parameters

##### note\_details

[`NoteId`](NoteId.md)

#### Returns

`NoteFile`

***

### fromOutputNote()

> `static` **fromOutputNote**(`note`): `NoteFile`

Creates a `NoteFile` from an output note, choosing details when present.

#### Parameters

##### note

[`OutputNote`](OutputNote.md)

#### Returns

`NoteFile`
