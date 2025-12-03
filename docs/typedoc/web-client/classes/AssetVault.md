[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / AssetVault

# Class: AssetVault

Sparse Merkle tree of assets held by an account.

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

> **fungibleAssets**(): [`FungibleAsset`](FungibleAsset.md)[]

Returns the fungible assets contained in this vault.

#### Returns

[`FungibleAsset`](FungibleAsset.md)[]

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
