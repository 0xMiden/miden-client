[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / FungibleAsset

# Class: FungibleAsset

Represents a fungible asset amount associated with a faucet account.

## Constructors

### Constructor

> **new FungibleAsset**(`faucet_id`, `amount`): `FungibleAsset`

Creates a new fungible asset reference for the provided faucet.

#### Parameters

##### faucet\_id

[`AccountId`](AccountId.md)

##### amount

`bigint`

#### Returns

`FungibleAsset`

## Methods

### amount()

> **amount**(): `bigint`

Returns the quantity of fungible assets.

#### Returns

`bigint`

***

### faucetId()

> **faucetId**(): [`AccountId`](AccountId.md)

Returns the faucet identifier that issued the asset.

#### Returns

[`AccountId`](AccountId.md)

***

### free()

> **free**(): `void`

#### Returns

`void`

***

### intoWord()

> **intoWord**(): [`Word`](Word.md)

Converts the asset into its raw word representation.

#### Returns

[`Word`](Word.md)
