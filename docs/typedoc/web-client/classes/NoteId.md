[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / NoteId

# Class: NoteId

## Constructors

### Constructor

> **new NoteId**(`recipient_digest`, `asset_commitment_digest`): `NoteId`

Builds a note ID from the recipient and asset commitments.

#### Parameters

##### recipient\_digest

[`Word`](Word.md)

##### asset\_commitment\_digest

[`Word`](Word.md)

#### Returns

`NoteId`

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

### toString()

> **toString**(): `string`

Returns the canonical hex representation of the note ID.

#### Returns

`string`

***

### fromHex()

> `static` **fromHex**(`hex`): `NoteId`

Parses a note ID from its hex encoding.

#### Parameters

##### hex

`string`

#### Returns

`NoteId`
