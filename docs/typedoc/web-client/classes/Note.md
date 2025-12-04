[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / Note

# Class: Note

Notes consist of note metadata and details. Note metadata is always public, but details may be
either public, encrypted, or private, depending on the note type. Note details consist of note
assets, script, inputs, and a serial number, the three latter grouped into a recipient object.

Note details can be reduced to two unique identifiers: [`NoteId`] and `Nullifier`. The former is
publicly associated with a note, while the latter is known only to entities which have access to
full note details.

Fungible and non-fungible asset transfers are done by moving assets to the note's assets. The
note's script determines the conditions required for the note consumption, i.e. the target
account of a P2ID or conditions of a SWAP, and the effects of the note. The serial number has a
double duty of preventing double spend, and providing unlikability to the consumer of a note.
The note's inputs allow for customization of its script.

To create a note, the kernel does not require all the information above, a user can create a
note only with the commitment to the script, inputs, the serial number (i.e., the recipient),
and the kernel only verifies the source account has the assets necessary for the note creation.
See [`NoteRecipient`] for more details.

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
