[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / StorageMapInfo

# Class: StorageMapInfo

Information about storage map updates for an account, as returned by the
`syncStorageMaps` RPC endpoint.

Contains the list of storage map updates within the requested block range,
along with the chain tip and last processed block number.

## Methods

### \[dispose\]()

> **\[dispose\]**(): `void`

#### Returns

`void`

***

### blockNumber()

> **blockNumber**(): `number`

Returns the block number of the last check included in this response.

#### Returns

`number`

***

### chainTip()

> **chainTip**(): `number`

Returns the current chain tip block number.

#### Returns

`number`

***

### free()

> **free**(): `void`

#### Returns

`void`

***

### updates()

> **updates**(): [`StorageMapUpdate`](StorageMapUpdate.md)[]

Returns the list of storage map updates.

#### Returns

[`StorageMapUpdate`](StorageMapUpdate.md)[]
