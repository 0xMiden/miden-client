---
title: AccountStorage
draft: true
---

[**@demox-labs/miden-sdk**](../index)

***

[@demox-labs/miden-sdk](../index) / AccountStorage

# Class: AccountStorage

## Methods

### commitment()

> **commitment**(): [`Word`](Word)

Returns a commitment to this storage.

#### Returns

[`Word`](Word)

***

### free()

> **free**(): `void`

#### Returns

`void`

***

### getItem()

> **getItem**(`index`): [`Word`](Word)

Returns an item from the storage at the specified index.

#### Parameters

##### index

`number`

The slot index in storage.

#### Returns

[`Word`](Word)

The stored `Word`, or `undefined` if not found.

#### Remarks

Errors:
- If the index is out of bounds

***

### getMapItem()

> **getMapItem**(`index`, `key`): [`Word`](Word)

Retrieves a map item from a map located in storage at the specified index.

#### Parameters

##### index

`number`

The slot index in storage.

##### key

[`Word`](Word)

The key used to look up the map item.

#### Returns

[`Word`](Word)

The stored `Word`, or `undefined` if not found.

#### Remarks

Errors:
- If the index is out of bounds
- If the indexed storage slot is not a map
