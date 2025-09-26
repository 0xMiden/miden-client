---
title: Note
draft: true
---

[**@demox-labs/miden-sdk**](../index)

***

[@demox-labs/miden-sdk](../index) / Note

# Class: Note

## Constructors

### Constructor

> **new Note**(`note_assets`, `note_metadata`, `note_recipient`): `Note`

#### Parameters

##### note\_assets

[`NoteAssets`](NoteAssets)

##### note\_metadata

[`NoteMetadata`](NoteMetadata)

##### note\_recipient

[`NoteRecipient`](NoteRecipient)

#### Returns

`Note`

## Methods

### assets()

> **assets**(): [`NoteAssets`](NoteAssets)

#### Returns

[`NoteAssets`](NoteAssets)

***

### free()

> **free**(): `void`

#### Returns

`void`

***

### id()

> **id**(): [`NoteId`](NoteId)

#### Returns

[`NoteId`](NoteId)

***

### metadata()

> **metadata**(): [`NoteMetadata`](NoteMetadata)

#### Returns

[`NoteMetadata`](NoteMetadata)

***

### recipient()

> **recipient**(): [`NoteRecipient`](NoteRecipient)

#### Returns

[`NoteRecipient`](NoteRecipient)

***

### script()

> **script**(): [`NoteScript`](NoteScript)

#### Returns

[`NoteScript`](NoteScript)

***

### serialize()

> **serialize**(): `Uint8Array`

#### Returns

`Uint8Array`

***

### createP2IDENote()

> `static` **createP2IDENote**(`sender`, `target`, `assets`, `reclaim_height`, `timelock_height`, `note_type`, `aux`): `Note`

#### Parameters

##### sender

[`AccountId`](AccountId)

##### target

[`AccountId`](AccountId)

##### assets

[`NoteAssets`](NoteAssets)

##### reclaim\_height

`number`

##### timelock\_height

`number`

##### note\_type

[`NoteType`](../enumerations/NoteType)

##### aux

[`Felt`](Felt)

#### Returns

`Note`

***

### createP2IDNote()

> `static` **createP2IDNote**(`sender`, `target`, `assets`, `note_type`, `aux`): `Note`

#### Parameters

##### sender

[`AccountId`](AccountId)

##### target

[`AccountId`](AccountId)

##### assets

[`NoteAssets`](NoteAssets)

##### note\_type

[`NoteType`](../enumerations/NoteType)

##### aux

[`Felt`](Felt)

#### Returns

`Note`

***

### deserialize()

> `static` **deserialize**(`bytes`): `Note`

#### Parameters

##### bytes

`Uint8Array`

#### Returns

`Note`
