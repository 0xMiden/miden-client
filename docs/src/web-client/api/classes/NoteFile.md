[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / NoteFile

# Class: NoteFile

## Methods

### free()

> **free**(): `void`

#### Returns

`void`

***

### noteType()

> **noteType**(): `string`

Returns this `NoteFile`'s types.

#### Returns

`string`

***

### serialize()

> **serialize**(): `Uint8Array`

Turn a notefile into its byte representation.

#### Returns

`Uint8Array`

***

### toJSON()

> **toJSON**(): `Object`

* Return copy of self without private attributes.

#### Returns

`Object`

***

### toString()

> **toString**(): `string`

Return stringified version of self.

#### Returns

`string`

***

### deserialize()

> `static` **deserialize**(`bytes`): `NoteFile`

Given a valid byte representation of a `NoteFile`,
return it as a struct.

#### Parameters

##### bytes

`Uint8Array`

#### Returns

`NoteFile`
