[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / FungibleAsset

# Class: FungibleAsset

## Constructors

### Constructor

> **new FungibleAsset**(`faucet_id`, `amount`): `FungibleAsset`

Creates a fungible asset for the given faucet and amount.

#### Parameters

##### faucet\_id

[`AccountId`](AccountId.md)

##### amount

`bigint`

#### Returns

`FungibleAsset`

## Methods

### \[dispose\]()

> **\[dispose\]**(): `void`

#### Returns

`void`

***

### amount()

> **amount**(): `bigint`

Returns the amount of fungible units.

#### Returns

`bigint`

***

### faucetId()

> **faucetId**(): [`AccountId`](AccountId.md)

Returns the faucet account that minted this asset.

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

Encodes this asset into the word layout used in the vault.

#### Returns

[`Word`](Word.md)
