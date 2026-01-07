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

### afterBlockNum()

> **afterBlockNum**(): `number`

Returns the after-block hint when present.

#### Returns

`number`

***

### free()

> **free**(): `void`

#### Returns

`void`

***

### inclusionProof()

> **inclusionProof**(): [`NoteInclusionProof`](NoteInclusionProof.md)

Returns the inclusion proof if present.

#### Returns

[`NoteInclusionProof`](NoteInclusionProof.md)

***

### note()

> **note**(): [`Note`](Note.md)

Returns the full note when the file includes it.

#### Returns

[`Note`](Note.md)

***

### noteDetails()

> **noteDetails**(): [`NoteDetails`](NoteDetails.md)

Returns the note details if present.

#### Returns

[`NoteDetails`](NoteDetails.md)

***

### noteId()

> **noteId**(): [`NoteId`](NoteId.md)

Returns the note ID for any `NoteFile` variant.

#### Returns

[`NoteId`](NoteId.md)

***

### noteTag()

> **noteTag**(): [`NoteTag`](NoteTag.md)

Returns the note tag hint when present.

#### Returns

[`NoteTag`](NoteTag.md)

***

### noteType()

> **noteType**(): `string`

Returns this `NoteFile`'s types.

#### Returns

`string`

***

### nullifier()

> **nullifier**(): `string`

Returns the note nullifier when present.

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
