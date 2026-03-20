[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / StorageMapUpdate

# Class: StorageMapUpdate

A single storage map update entry, containing the block number, slot name,
key, and new value.

## Methods

### \[dispose\]()

> **\[dispose\]**(): `void`

#### Returns

`void`

***

### blockNum()

> **blockNum**(): `number`

Returns the block number in which this update occurred.

#### Returns

`number`

***

### free()

> **free**(): `void`

#### Returns

`void`

***

### key()

> **key**(): [`Word`](Word.md)

Returns the storage map key that was updated.

#### Returns

[`Word`](Word.md)

***

### slotName()

> **slotName**(): `string`

Returns the name of the storage slot that was updated.

#### Returns

`string`

***

### value()

> **value**(): [`Word`](Word.md)

Returns the new value for this storage map key.

#### Returns

[`Word`](Word.md)
