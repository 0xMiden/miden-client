[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / AccountStorageDelta

# Class: AccountStorageDelta

Wrapper around [`miden_client::asset::AccountStorageDelta`].

Describes changes applied to an account's storage slots.

## Methods

### free()

> **free**(): `void`

#### Returns

`void`

***

### isEmpty()

> **isEmpty**(): `boolean`

Returns `true` if the delta does not change any slots.

#### Returns

`boolean`

***

### serialize()

> **serialize**(): `Uint8Array`

#### Returns

`Uint8Array`

***

### values()

> **values**(): [`Word`](Word.md)[]

Returns the values written to storage slots as field elements.

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

#### Throws

Throws if the bytes are invalid.
