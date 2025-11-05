[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / ConsumableNoteRecord

# Class: ConsumableNoteRecord

Note together with metadata describing when each account may consume it.

## Constructors

### Constructor

> **new ConsumableNoteRecord**(`input_note_record`, `note_consumability`): `ConsumableNoteRecord`

Creates a consumable note record with explicit consumability metadata.

#### Parameters

##### input\_note\_record

[`InputNoteRecord`](InputNoteRecord.md)

##### note\_consumability

[`NoteConsumability`](NoteConsumability.md)[]

#### Returns

`ConsumableNoteRecord`

## Methods

### free()

> **free**(): `void`

#### Returns

`void`

***

### inputNoteRecord()

> **inputNoteRecord**(): [`InputNoteRecord`](InputNoteRecord.md)

Returns the underlying input note record.

#### Returns

[`InputNoteRecord`](InputNoteRecord.md)

***

### noteConsumability()

> **noteConsumability**(): [`NoteConsumability`](NoteConsumability.md)[]

Returns per-account consumability entries for the note.

#### Returns

[`NoteConsumability`](NoteConsumability.md)[]
