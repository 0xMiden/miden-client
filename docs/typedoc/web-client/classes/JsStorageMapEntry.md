[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / JsStorageMapEntry

# Class: JsStorageMapEntry

A JavaScript representation of a storage map entry in an account.

## Properties

### accountId

> **accountId**: `string`

The account ID this map entry belongs to.

***

### key

> **key**: `string`

The key of the storage map entry.

***

### nonce

> **nonce**: `string`

The account's nonce when this entry was recorded.

***

### slotName

> **slotName**: `string`

The slot name of the map this entry belongs to.

***

### value

> **value**: `string`

The value of the storage map entry.

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
