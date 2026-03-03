[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / ConsumableNoteRecord

# Class: ConsumableNoteRecord

Input note record annotated with consumption conditions.

## Constructors

### Constructor

> **new ConsumableNoteRecord**(`input_note_record`, `note_consumability`): `ConsumableNoteRecord`

Creates a new consumable note record from an input note record and consumability metadata.

#### Parameters

##### input\_note\_record

[`InputNoteRecord`](InputNoteRecord.md)

##### note\_consumability

`NoteConsumability`[]

#### Returns

`ConsumableNoteRecord`

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

### inputNoteRecord()

> **inputNoteRecord**(): [`InputNoteRecord`](InputNoteRecord.md)

Returns the underlying input note record.

#### Returns

[`InputNoteRecord`](InputNoteRecord.md)

***

### noteConsumability()

> **noteConsumability**(): `NoteConsumability`[]

Returns the consumability entries.

#### Returns

`NoteConsumability`[]
