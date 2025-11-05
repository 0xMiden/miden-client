[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / AccountStorageMode

# Class: AccountStorageMode

Storage mode configuration for an account (private, public, or network).

## Methods

### asStr()

> **asStr**(): `string`

Returns the string representation of the storage mode.

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

Returns the network storage mode, where storage is managed by the network.

#### Returns

`AccountStorageMode`

***

### private()

> `static` **private**(): `AccountStorageMode`

Returns the private storage mode, where data stays local to the client.

#### Returns

`AccountStorageMode`

***

### public()

> `static` **public**(): `AccountStorageMode`

Returns the public storage mode, where data is fully public.

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

#### Throws

Throws if the provided string does not match a known mode.
