[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / SparseMerklePath

# Class: SparseMerklePath

Represents a sparse Merkle path.

## Methods

### \[dispose\]()

> **\[dispose\]**(): `void`

#### Returns

`void`

***

### emptyNodesMask()

> **emptyNodesMask**(): `bigint`

Returns the empty nodes mask used by this path.

#### Returns

`bigint`

***

### free()

> **free**(): `void`

#### Returns

`void`

***

### nodes()

> **nodes**(): [`Word`](Word.md)[]

Returns the sibling nodes that make up the path.

#### Returns

[`Word`](Word.md)[]

***

### verify()

> **verify**(`index`, `node`, `root`): `boolean`

Verifies the path against a root.

#### Parameters

##### index

`bigint`

##### node

[`Word`](Word.md)

##### root

[`Word`](Word.md)

#### Returns

`boolean`
