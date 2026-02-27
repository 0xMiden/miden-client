[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / OutputNoteRecord

# Class: OutputNoteRecord

Represents an output note tracked by the client store.

## Methods

### \[dispose\]()

> **\[dispose\]**(): `void`

#### Returns

`void`

***

### assets()

> **assets**(): `NoteAssets`

Returns the note assets.

#### Returns

`NoteAssets`

***

### expectedHeight()

> **expectedHeight**(): `number`

Returns the expected block height for the note.

#### Returns

`number`

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

Returns the inclusion proof when the note is committed.

#### Returns

`NoteInclusionProof`

***

### isCommitted()

> **isCommitted**(): `boolean`

Returns true if the note is committed on chain.

#### Returns

`boolean`

***

### isConsumed()

> **isConsumed**(): `boolean`

Returns true if the note has been consumed on chain.

#### Returns

`boolean`

***

### metadata()

> **metadata**(): `NoteMetadata`

Returns the note metadata.

#### Returns

`NoteMetadata`

***

### nullifier()

> **nullifier**(): `string`

Returns the nullifier when the recipient is known.

#### Returns

`string`

***

### recipient()

> **recipient**(): `NoteRecipient`

Returns the recipient details if available.

#### Returns

`NoteRecipient`

***

### recipientDigest()

> **recipientDigest**(): [`Word`](Word.md)

Returns the recipient digest committed for the note.

#### Returns

[`Word`](Word.md)

***

### state()

> **state**(): `OutputNoteState`

Returns the current processing state for this note.

#### Returns

`OutputNoteState`
