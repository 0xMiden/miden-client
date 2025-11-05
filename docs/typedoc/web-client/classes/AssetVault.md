[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / AssetVault

# Class: AssetVault

Representation of an account's asset vault exposed to JavaScript.

## Methods

### free()

> **free**(): `void`

#### Returns

`void`

***

### fungibleAssets()

> **fungibleAssets**(): [`FungibleAsset`](FungibleAsset.md)[]

Returns all fungible assets stored in the vault.

#### Returns

[`FungibleAsset`](FungibleAsset.md)[]

***

### getBalance()

> **getBalance**(`faucet_id`): `bigint`

Returns the balance for the provided faucet identifier.

#### Parameters

##### faucet\_id

[`AccountId`](AccountId.md)

#### Returns

`bigint`

***

### root()

> **root**(): [`Word`](Word.md)

Returns the Merkle root commitment for the vault contents.

#### Returns

[`Word`](Word.md)
