[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / FungibleAssetDeltaItem

# Class: FungibleAssetDeltaItem

Single entry of a fungible asset delta, containing faucet ID and delta amount.

## Properties

### amount

> `readonly` **amount**: `bigint`

Returns the signed balance change (positive adds assets, negative removes).

***

### faucetId

> `readonly` **faucetId**: [`AccountId`](AccountId.md)

Returns the faucet identifier associated with this delta entry.

## Methods

### free()

> **free**(): `void`

#### Returns

`void`
