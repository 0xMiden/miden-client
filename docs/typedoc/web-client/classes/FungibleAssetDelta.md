[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / FungibleAssetDelta

# Class: FungibleAssetDelta

Aggregated changes to fungible asset balances.

## Methods

### amount()

> **amount**(`faucet_id`): `bigint`

Returns the signed change for the given faucet, if present.

#### Parameters

##### faucet\_id

[`AccountId`](AccountId.md)

#### Returns

`bigint`

***

### assets()

> **assets**(): [`FungibleAssetDeltaItem`](FungibleAssetDeltaItem.md)[]

Returns all faucet deltas contained in this structure.

#### Returns

[`FungibleAssetDeltaItem`](FungibleAssetDeltaItem.md)[]

***

### free()

> **free**(): `void`

#### Returns

`void`

***

### isEmpty()

> **isEmpty**(): `boolean`

Returns `true` if no balances changed.

#### Returns

`boolean`

***

### numAssets()

> **numAssets**(): `number`

Returns the number of faucet entries tracked in this delta.

#### Returns

`number`

***

### serialize()

> **serialize**(): `Uint8Array`

#### Returns

`Uint8Array`

***

### deserialize()

> `static` **deserialize**(`bytes`): `FungibleAssetDelta`

Deserializes a fungible asset delta from bytes.

#### Parameters

##### bytes

`Uint8Array`

#### Returns

`FungibleAssetDelta`

#### Throws

Throws if the bytes are invalid.
