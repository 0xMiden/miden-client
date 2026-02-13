[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / JsStorageSlot

# Class: JsStorageSlot

A JavaScript representation of a storage slot in an account.

## Properties

### accountId

> **accountId**: `string`

The account ID this slot belongs to.

***

### nonce

> **nonce**: `string`

The account's nonce when this slot state was recorded.

***

### slotName

> **slotName**: `string`

The name of the storage slot.

***

### slotType

> **slotType**: `number`

The type of the storage slot.

***

### slotValue

> **slotValue**: `string`

The value stored in the storage slot.

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
