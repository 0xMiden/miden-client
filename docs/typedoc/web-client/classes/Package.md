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

### free()

> **free**(): `void`

#### Returns

`void`

***

### serialize()

> **serialize**(): `Uint8Array`

#### Returns

`Uint8Array`

***

### tryAsLibrary()

> **tryAsLibrary**(): [`Library`](Library.md)

Returns the underlying library of a `Package`.
Fails if the package is not a library.

#### Returns

[`Library`](Library.md)

***

### tryAsProgram()

> **tryAsProgram**(): [`Program`](Program.md)

Returns the underlying program of a `Package`.
Fails if the package is not a program.

#### Returns

[`Program`](Program.md)

***

### deserialize()

> `static` **deserialize**(`bytes`): `Package`

#### Parameters

##### bytes

`Uint8Array`

#### Returns

`Package`
