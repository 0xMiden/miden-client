[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / InputNoteRecord

# Class: InputNoteRecord

Represents a Note of which the Store can keep track and retrieve.

An `InputNoteRecord` contains all the information of a `NoteDetails`, in addition to specific
information about the note state.

Once a proof is received, the `InputNoteRecord` can be transformed into an `InputNote` and used
as input for transactions. It is also possible to convert `Note` and `InputNote` into
`InputNoteRecord` (we fill the `metadata` and `inclusion_proof` fields if possible).

Notes can also be consumed as unauthenticated notes, where their existence is verified by the
network.

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

> **details**(): `NoteDetails`

Returns the note details, if present.

#### Returns

`NoteDetails`

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

> **inclusionProof**(): `NoteInclusionProof`

Returns the inclusion proof when the note is authenticated.

#### Returns

`NoteInclusionProof`

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

> **metadata**(): `NoteMetadata`

Returns the note metadata if available.

#### Returns

`NoteMetadata`

***

### nullifier()

> **nullifier**(): `string`

Returns the nullifier for this note.

#### Returns

`string`

***

### state()

> **state**(): `InputNoteState`

Returns the current processing state for this note.

#### Returns

`InputNoteState`

***

### toInputNote()

> **toInputNote**(): `InputNote`

Converts the record into an `InputNote` (including proof when available).

#### Returns

`InputNote`

***

### toNote()

> **toNote**(): [`Note`](Note.md)

Converts the record into a `Note` (including proof when available).

#### Returns

[`Note`](Note.md)
