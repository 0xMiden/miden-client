[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / FungibleAssetDelta

# Class: FungibleAssetDelta

Aggregated fungible deltas keyed by faucet ID.

## Methods

### \[dispose\]()

> **\[dispose\]**(): `void`

#### Returns

`void`

***

### amount()

> **amount**(`faucet_id`): `bigint`

Returns the delta amount for a given faucet, if present.

#### Parameters

##### faucet\_id

[`AccountId`](AccountId.md)

#### Returns

`bigint`

***

### assets()

> **assets**(): [`FungibleAssetDeltaItem`](FungibleAssetDeltaItem.md)[]

Returns all fungible asset deltas as a list.

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

Returns true if no fungible assets are affected.

#### Returns

`boolean`

***

### numAssets()

> **numAssets**(): `number`

Returns the number of distinct fungible assets in the delta.

#### Returns

`number`

***

### serialize()

> **serialize**(): `Uint8Array`

Serializes the fungible delta into bytes.

#### Returns

`Uint8Array`

***

### deserialize()

> `static` **deserialize**(`bytes`): `FungibleAssetDelta`

Deserializes a fungible delta from bytes.

#### Parameters

##### bytes

`Uint8Array`

#### Returns

`FungibleAssetDelta`
