[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / Package

# Class: Package

## Methods

### \[dispose\]()

> **\[dispose\]**(): `void`

#### Returns

`void`

***

### asLibrary()

> **asLibrary**(): [`Library`](Library.md)

Returns the underlying library of a `Package`.
Fails if the package is not a library.

#### Returns

[`Library`](Library.md)

***

### asProgram()

> **asProgram**(): [`Program`](Program.md)

Returns the underlying program of a `Package`.
Fails if the package is not a program.

#### Returns

[`Program`](Program.md)

***

### free()

> **free**(): `void`

#### Returns

`void`

***

### serialize()

> **serialize**(): `Uint8Array`

Serializes the package into bytes.

#### Returns

`Uint8Array`

***

### deserialize()

> `static` **deserialize**(`bytes`): `Package`

Deserializes a package from bytes.

#### Parameters

##### bytes

`Uint8Array`

#### Returns

`Package`
