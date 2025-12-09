[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / NoteScript

# Class: NoteScript

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

### root()

> **root**(): [`Word`](Word.md)

Returns the MAST root of this script.

#### Returns

[`Word`](Word.md)

***

### serialize()

> **serialize**(): `Uint8Array`

Serializes the script into bytes.

#### Returns

`Uint8Array`

***

### toString()

> **toString**(): `string`

Pretty-prints the MAST source for this script.

#### Returns

`string`

***

### deserialize()

> `static` **deserialize**(`bytes`): `NoteScript`

Deserializes a script from bytes.

#### Parameters

##### bytes

`Uint8Array`

#### Returns

`NoteScript`

***

### fromPackage()

> `static` **fromPackage**(`_package`): `NoteScript`

Creates a `NoteScript` from the given `Package`.
Throws if the package is invalid.

#### Parameters

##### \_package

[`Package`](Package.md)

#### Returns

`NoteScript`

***

### p2id()

> `static` **p2id**(): `NoteScript`

Returns the well-known P2ID script.

#### Returns

`NoteScript`

***

### p2ide()

> `static` **p2ide**(): `NoteScript`

Returns the well-known P2IDE script (P2ID with execution hint).

#### Returns

`NoteScript`

***

### swap()

> `static` **swap**(): `NoteScript`

Returns the well-known SWAP script.

#### Returns

`NoteScript`
