[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / NoteAttachmentScheme

# Class: NoteAttachmentScheme

Describes the type of a note attachment.

Value `0` is reserved to signal that the scheme is none or absent. Whenever the kind of
attachment is not standardized or interoperability is unimportant, this none value can be used.

## Constructors

### Constructor

> **new NoteAttachmentScheme**(`scheme`): `NoteAttachmentScheme`

Creates a new `NoteAttachmentScheme` from a u32.

#### Parameters

##### scheme

`number`

#### Returns

`NoteAttachmentScheme`

## Methods

### \[dispose\]()

> **\[dispose\]**(): `void`

#### Returns

`void`

***

### asU32()

> **asU32**(): `number`

Returns the note attachment scheme as a u32.

#### Returns

`number`

***

### free()

> **free**(): `void`

#### Returns

`void`

***

### isNone()

> **isNone**(): `boolean`

Returns true if the attachment scheme is the reserved value that signals an absent scheme.

#### Returns

`boolean`

***

### none()

> `static` **none**(): `NoteAttachmentScheme`

Returns the `NoteAttachmentScheme` that signals the absence of an attachment scheme.

#### Returns

`NoteAttachmentScheme`
