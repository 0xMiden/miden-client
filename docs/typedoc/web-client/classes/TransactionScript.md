[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / TransactionScript

# Class: TransactionScript

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

Returns the MAST root commitment of the transaction script.

#### Returns

[`Word`](Word.md)

***

### fromPackage()

> `static` **fromPackage**(`_package`): `TransactionScript`

Creates a `NoteScript` from the given `Package`.
Throws if the package is invalid.

#### Parameters

##### \_package

[`Package`](Package.md)

#### Returns

`TransactionScript`
