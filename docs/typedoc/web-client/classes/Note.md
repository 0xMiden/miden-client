[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / Note

# Class: Note

A note bundles public metadata with private details: assets, script, inputs, and a serial number
grouped into a recipient. The public identifier (`NoteId`) commits to those
details, while the nullifier stays hidden until the note is consumed. Assets move by
transferring them into the note; the script and inputs define how and when consumption can
happen. See `NoteRecipient` for the shape of the recipient data.

## Constructors

### Constructor

> **new Note**(`note_assets`, `note_metadata`, `note_recipient`): `Note`

Creates a new note from the provided assets, metadata, and recipient.

#### Parameters

##### note\_assets

[`NoteAssets`](NoteAssets.md)

##### note\_metadata

[`NoteMetadata`](NoteMetadata.md)

##### note\_recipient

[`NoteRecipient`](NoteRecipient.md)

#### Returns

`Note`

## Methods

### \[dispose\]()

> **\[dispose\]**(): `void`

#### Returns

`void`

***

### assets()

> **assets**(): [`NoteAssets`](NoteAssets.md)

Returns the assets locked inside the note.

#### Returns

[`NoteAssets`](NoteAssets.md)

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

Returns the unique identifier of the note.

#### Returns

[`NoteId`](NoteId.md)

***

### metadata()

> **metadata**(): [`NoteMetadata`](NoteMetadata.md)

Returns the public metadata associated with the note.

#### Returns

[`NoteMetadata`](NoteMetadata.md)

***

### recipient()

> **recipient**(): [`NoteRecipient`](NoteRecipient.md)

Returns the recipient who can consume this note.

#### Returns

[`NoteRecipient`](NoteRecipient.md)

***

### script()

> **script**(): [`NoteScript`](NoteScript.md)

Returns the script that guards the note.

#### Returns

[`NoteScript`](NoteScript.md)

***

### serialize()

> **serialize**(): `Uint8Array`

Serializes the note into bytes.

#### Returns

`Uint8Array`

***

### createP2IDENote()

> `static` **createP2IDENote**(`sender`, `target`, `assets`, `reclaim_height`, `timelock_height`, `note_type`, `aux`): `Note`

Builds a P2IDE note that can be reclaimed or timelocked based on block heights.

#### Parameters

##### sender

[`AccountId`](AccountId.md)

##### target

[`AccountId`](AccountId.md)

##### assets

[`NoteAssets`](NoteAssets.md)

##### reclaim\_height

`number`

##### timelock\_height

`number`

##### note\_type

[`NoteType`](../enumerations/NoteType.md)

##### aux

[`Felt`](Felt.md)

#### Returns

`Note`

***

### createP2IDNote()

> `static` **createP2IDNote**(`sender`, `target`, `assets`, `note_type`, `aux`): `Note`

Builds a standard P2ID note that targets the specified account.

#### Parameters

##### sender

[`AccountId`](AccountId.md)

##### target

[`AccountId`](AccountId.md)

##### assets

[`NoteAssets`](NoteAssets.md)

##### note\_type

[`NoteType`](../enumerations/NoteType.md)

##### aux

[`Felt`](Felt.md)

#### Returns

`Note`

***

### deserialize()

> `static` **deserialize**(`bytes`): `Note`

Deserializes a note from its byte representation.

#### Parameters

##### bytes

`Uint8Array`

#### Returns

`Note`
