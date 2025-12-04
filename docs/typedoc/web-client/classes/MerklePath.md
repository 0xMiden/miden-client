[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / MerklePath

# Class: MerklePath

Represents a Merkle path.

## Methods

### \[dispose\]()

> **\[dispose\]**(): `void`

#### Returns

`void`

***

### computeRoot()

> **computeRoot**(`index`, `node`): [`Word`](Word.md)

Computes the root given a leaf index and value.

#### Parameters

##### index

`bigint`

##### node

[`Word`](Word.md)

#### Returns

[`Word`](Word.md)

***

### depth()

> **depth**(): `number`

Returns the depth of the path.

#### Returns

`number`

***

### free()

> **free**(): `void`

#### Returns

`void`

***

### nodes()

> **nodes**(): [`Word`](Word.md)[]

Returns the nodes that make up the path.

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
