[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / StorageMap

# Class: StorageMap

Key/value storage map used within accounts.

## Constructors

### Constructor

> **new StorageMap**(): `StorageMap`

Creates an empty storage map.

#### Returns

`StorageMap`

## Methods

### free()

> **free**(): `void`

#### Returns

`void`

***

### insert()

> **insert**(`key`, `value`): [`Word`](Word.md)

Inserts or updates a key and returns the previous value (or zero if absent).

#### Parameters

##### key

[`Word`](Word.md)

##### value

[`Word`](Word.md)

#### Returns

[`Word`](Word.md)
