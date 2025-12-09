[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / InputNoteRecord

# Class: InputNoteRecord

## Methods

### \[dispose\]()

> **\[dispose\]**(): `void`

#### Returns

`void`

***

### commitment()

> **commitment**(): [`Word`](Word.md)

Returns the note commitment (id + metadata), if available.

#### Returns

[`Word`](Word.md)

***

### consumerTransactionId()

> **consumerTransactionId**(): `string`

Returns the transaction ID that consumed this note, if any.

#### Returns

`string`

***

### details()

> **details**(): [`NoteDetails`](NoteDetails.md)

Returns the note details, if present.

#### Returns

[`NoteDetails`](NoteDetails.md)

***

### free()

> **free**(): `void`

#### Returns

`void`

***

### id()

> **id**(): [`NoteId`](NoteId.md)

Returns the note ID.

#### Returns

[`NoteId`](NoteId.md)

***

### inclusionProof()

> **inclusionProof**(): [`NoteInclusionProof`](NoteInclusionProof.md)

Returns the inclusion proof when the note is authenticated.

#### Returns

[`NoteInclusionProof`](NoteInclusionProof.md)

***

### isAuthenticated()

> **isAuthenticated**(): `boolean`

Returns true if the record contains authentication data (proof).

#### Returns

`boolean`

***

### isConsumed()

> **isConsumed**(): `boolean`

Returns true if the note has already been consumed.

#### Returns

`boolean`

***

### isProcessing()

> **isProcessing**(): `boolean`

Returns true if the note is currently being processed.

#### Returns

`boolean`

***

### metadata()

> **metadata**(): [`NoteMetadata`](NoteMetadata.md)

Returns the note metadata if available.

#### Returns

[`NoteMetadata`](NoteMetadata.md)

***

### nullifier()

> **nullifier**(): `string`

Returns the nullifier for this note.

#### Returns

`string`

***

### state()

> **state**(): [`InputNoteState`](../enumerations/InputNoteState.md)

Returns the current processing state for this note.

#### Returns

[`InputNoteState`](../enumerations/InputNoteState.md)

***

### toInputNote()

> **toInputNote**(): [`InputNote`](InputNote.md)

Converts the record into an `InputNote` (including proof when available).

#### Returns

[`InputNote`](InputNote.md)
