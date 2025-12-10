[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / InputNote

# Class: InputNote

Note supplied as an input to a transaction, optionally with authentication data.

## Methods

### \[dispose\]()

> **\[dispose\]**(): `void`

#### Returns

`void`

***

### commitment()

> **commitment**(): [`Word`](Word.md)

Returns the commitment to the note ID and metadata.

#### Returns

[`Word`](Word.md)

***

### free()

> **free**(): `void`

#### Returns

`void`

***

### id()

> **id**(): [`NoteId`](NoteId.md)

Returns the identifier of the input note.

#### Returns

[`NoteId`](NoteId.md)

***

### location()

> **location**(): [`NoteLocation`](NoteLocation.md)

Returns the note's location within the commitment tree when available.

#### Returns

[`NoteLocation`](NoteLocation.md)

***

### note()

> **note**(): [`Note`](Note.md)

Returns the underlying note contents.

#### Returns

[`Note`](Note.md)

***

### proof()

> **proof**(): [`NoteInclusionProof`](NoteInclusionProof.md)

Returns the inclusion proof if the note is authenticated.

#### Returns

[`NoteInclusionProof`](NoteInclusionProof.md)

***

### authenticated()

> `static` **authenticated**(`note`, `inclusion_proof`): `InputNote`

Creates an authenticated input note from a note and its inclusion proof.

An authenticated note has a proof of inclusion in the block's note tree,
which is required for consuming the note in a transaction.

#### Parameters

##### note

[`Note`](Note.md)

##### inclusion\_proof

[`NoteInclusionProof`](NoteInclusionProof.md)

#### Returns

`InputNote`

***

### unauthenticated()

> `static` **unauthenticated**(`note`): `InputNote`

Creates an unauthenticated input note from note details.

An unauthenticated note can be consumed in a transaction as long as the note exists in the
network as of the transaction batch in which the consume transaction is included.

#### Parameters

##### note

[`Note`](Note.md)

#### Returns

`InputNote`
