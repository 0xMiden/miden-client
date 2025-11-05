[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / NoteDetails

# Class: NoteDetails

Note details used in transaction requests and note stores.

## Constructors

### Constructor

> **new NoteDetails**(`note_assets`, `note_recipient`): `NoteDetails`

Creates new note details from assets and recipient.

#### Parameters

##### note\_assets

[`NoteAssets`](NoteAssets.md)

##### note\_recipient

[`NoteRecipient`](NoteRecipient.md)

#### Returns

`NoteDetails`

## Methods

### assets()

> **assets**(): [`NoteAssets`](NoteAssets.md)

Returns the assets locked in the note.

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

### recipient()

> **recipient**(): [`NoteRecipient`](NoteRecipient.md)

Returns the note recipient descriptor.

#### Returns

[`NoteRecipient`](NoteRecipient.md)
