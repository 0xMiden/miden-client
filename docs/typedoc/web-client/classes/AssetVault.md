[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / AssetVault

# Class: AssetVault

A container for an unlimited number of assets.

An asset vault can contain an unlimited number of assets. The assets are stored in a Sparse
Merkle tree as follows:
- For fungible assets, the index of a node is defined by the issuing faucet ID, and the value of
  the node is the asset itself. Thus, for any fungible asset there will be only one node in the
  tree.
- For non-fungible assets, the index is defined by the asset itself, and the asset is also the
  value of the node.

An asset vault can be reduced to a single hash which is the root of the Sparse Merkle Tree.

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

### fungibleAssets()

> **fungibleAssets**(): `FungibleAsset`[]

Returns the fungible assets contained in this vault.

#### Returns

`FungibleAsset`[]

***

### getBalance()

> **getBalance**(`faucet_id`): `bigint`

Returns the balance for the given fungible faucet, or zero if absent.

#### Parameters

##### faucet\_id

[`AccountId`](AccountId.md)

#### Returns

`bigint`

***

### root()

> **root**(): [`Word`](Word.md)

Returns the root commitment of the asset vault tree.

#### Returns

[`Word`](Word.md)
