[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / NoteId

# Class: NoteId

Returns a unique identifier of a note, which is simultaneously a commitment to the note.

Note ID is computed as:

> `hash(recipient, asset_commitment)`

where `recipient` is defined as:

> `hash(hash(hash(serial_num, ZERO), script_root), input_commitment)`

This achieves the following properties:
- Every note can be reduced to a single unique ID.
- To compute a note ID, we do not need to know the note's `serial_num`. Knowing the hash of the
  `serial_num` (as well as script root, input commitment, and note assets) is sufficient.

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
