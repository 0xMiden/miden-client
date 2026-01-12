[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / NoteDetails

# Class: NoteDetails

Details of a note consisting of assets, script, inputs, and a serial number.

See the [Note](Note.md) type for more details.

## Constructors

### Constructor

> **new NoteDetails**(`note_assets`, `note_recipient`): `NoteDetails`

Creates a new set of note details from the given assets and recipient.

#### Parameters

##### note\_assets

[`NoteAssets`](NoteAssets.md)

##### note\_recipient

[`NoteRecipient`](NoteRecipient.md)

#### Returns

`NoteDetails`

## Methods

### \[dispose\]()

> **\[dispose\]**(): `void`

#### Returns

`void`

***

### assets()

> **assets**(): [`NoteAssets`](NoteAssets.md)

Returns the assets locked by the note.

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

Returns the note identifier derived from these details.

#### Returns

[`NoteId`](NoteId.md)

***

### nullifier()

> **nullifier**(): [`Word`](Word.md)

Returns the note nullifier as a word.

#### Returns

[`Word`](Word.md)

***

### recipient()

> **recipient**(): [`NoteRecipient`](NoteRecipient.md)

Returns the recipient which controls when the note can be consumed.

#### Returns

[`NoteRecipient`](NoteRecipient.md)
