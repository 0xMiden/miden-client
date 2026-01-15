[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / AccountStorageMode

# Class: AccountStorageMode

Storage visibility mode for an account.

## Methods

### \[dispose\]()

> **\[dispose\]**(): `void`

#### Returns

`void`

***

### asStr()

> **asStr**(): `string`

Returns the storage mode as a string.

#### Returns

`string`

***

### free()

> **free**(): `void`

#### Returns

`void`

***

### network()

> `static` **network**(): `AccountStorageMode`

Creates a network storage mode.

#### Returns

`AccountStorageMode`

***

### private()

> `static` **private**(): `AccountStorageMode`

Creates a private storage mode.

#### Returns

`AccountStorageMode`

***

### public()

> `static` **public**(): `AccountStorageMode`

Creates a public storage mode.

#### Returns

`AccountStorageMode`

***

### tryFromStr()

> `static` **tryFromStr**(`s`): `AccountStorageMode`

Parses a storage mode from its string representation.

#### Parameters

##### s

`string`

#### Returns

`AccountStorageMode`
