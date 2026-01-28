[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / NoteSyncInfo

# Class: NoteSyncInfo

Represents the response data from `syncNotes`.

## Methods

### \[dispose\]()

> **\[dispose\]**(): `void`

#### Returns

`void`

***

### blockHeader()

> **blockHeader**(): [`BlockHeader`](BlockHeader.md)

Returns the block header associated with the matching notes.

#### Returns

[`BlockHeader`](BlockHeader.md)

***

### chainTip()

> **chainTip**(): `number`

Returns the latest block number in the chain.

#### Returns

`number`

***

### free()

> **free**(): `void`

#### Returns

`void`

***

### mmrPath()

> **mmrPath**(): [`MerklePath`](MerklePath.md)

Returns the MMR path for the block header.

#### Returns

[`MerklePath`](MerklePath.md)

***

### notes()

> **notes**(): [`CommittedNote`](CommittedNote.md)[]

Returns the committed notes returned by the node.

#### Returns

[`CommittedNote`](CommittedNote.md)[]
