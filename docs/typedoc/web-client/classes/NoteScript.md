[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / NoteScript

# Class: NoteScript

Executable script that governs when and how a note can be consumed.

## Methods

### free()

> **free**(): `void`

#### Returns

`void`

***

### root()

> **root**(): [`Word`](Word.md)

Returns the MAST root hash of the script.

#### Returns

[`Word`](Word.md)

***

### serialize()

> **serialize**(): `Uint8Array`

Serializes the note script into bytes.

#### Returns

`Uint8Array`

***

### toString()

> **toString**(): `string`

Print the MAST source for this script.

#### Returns

`string`

***

### deserialize()

> `static` **deserialize**(`bytes`): `NoteScript`

Deserializes a note script from bytes.

#### Parameters

##### bytes

`Uint8Array`

#### Returns

`NoteScript`

***

### p2id()

> `static` **p2id**(): `NoteScript`

Returns the well-known pay-to-identity note script.

#### Returns

`NoteScript`

***

### p2ide()

> `static` **p2ide**(): `NoteScript`

Returns the well-known pay-to-identity with embedded conditions script.

#### Returns

`NoteScript`

***

### swap()

> `static` **swap**(): `NoteScript`

Returns the built-in swap note script.

#### Returns

`NoteScript`
