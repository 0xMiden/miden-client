[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / Note

# Class: Note

Wrapper around a note, the fundamental unit of value transfer in Miden.

## Constructors

### Constructor

> **new Note**(`note_assets`, `note_metadata`, `note_recipient`): `Note`

Creates a new note from assets, metadata, and recipient information.

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

### assets()

> **assets**(): [`NoteAssets`](NoteAssets.md)

Returns the assets locked into the note.

#### Returns

[`NoteAssets`](NoteAssets.md)

***

### free()

> **free**(): `void`

#### Returns

`void`

***

### id()

> **id**(): [`NoteId`](NoteId.md)

Returns the note identifier.

#### Returns

[`NoteId`](NoteId.md)

***

### metadata()

> **metadata**(): [`NoteMetadata`](NoteMetadata.md)

Returns the note metadata.

#### Returns

[`NoteMetadata`](NoteMetadata.md)

***

### recipient()

> **recipient**(): [`NoteRecipient`](NoteRecipient.md)

Returns the note recipient.

#### Returns

[`NoteRecipient`](NoteRecipient.md)

***

### script()

> **script**(): [`NoteScript`](NoteScript.md)

Returns the script that governs note consumption.

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

Creates a pay-to-identity-with-embedded-conditions note.

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

Creates a pay-to-identity note with a random blinding coin.

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

Deserializes a note from bytes produced by [`serialize`].

#### Parameters

##### bytes

`Uint8Array`

#### Returns

`Note`
