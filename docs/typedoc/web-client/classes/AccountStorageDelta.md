[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / AccountStorageDelta

# Class: AccountStorageDelta

Difference to apply to account storage slots.

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

### isEmpty()

> **isEmpty**(): `boolean`

Returns true if no storage slots are changed.

#### Returns

`boolean`

***

### serialize()

> **serialize**(): `Uint8Array`

Serializes the storage delta into bytes.

#### Returns

`Uint8Array`

***

### values()

> **values**(): [`Word`](Word.md)[]

Returns the new values for modified storage slots.

#### Returns

[`Word`](Word.md)[]

***

### deserialize()

> `static` **deserialize**(`bytes`): `AccountStorageDelta`

Deserializes a storage delta from bytes.

#### Parameters

##### bytes

`Uint8Array`

#### Returns

`AccountStorageDelta`
