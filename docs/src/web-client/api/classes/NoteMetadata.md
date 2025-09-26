---
title: NoteMetadata
draft: true
---

[**@demox-labs/miden-sdk**](../index)

***

[@demox-labs/miden-sdk](../index) / NoteMetadata

# Class: NoteMetadata

## Constructors

### Constructor

> **new NoteMetadata**(`sender`, `note_type`, `note_tag`, `note_execution_hint`, `aux?`): `NoteMetadata`

#### Parameters

##### sender

[`AccountId`](AccountId)

##### note\_type

[`NoteType`](../enumerations/NoteType)

##### note\_tag

[`NoteTag`](NoteTag)

##### note\_execution\_hint

[`NoteExecutionHint`](NoteExecutionHint)

##### aux?

[`Felt`](Felt)

#### Returns

`NoteMetadata`

## Methods

### free()

> **free**(): `void`

#### Returns

`void`

***

### noteType()

> **noteType**(): [`NoteType`](../enumerations/NoteType)

#### Returns

[`NoteType`](../enumerations/NoteType)

***

### sender()

> **sender**(): [`AccountId`](AccountId)

#### Returns

[`AccountId`](AccountId)

***

### tag()

> **tag**(): [`NoteTag`](NoteTag)

#### Returns

[`NoteTag`](NoteTag)
