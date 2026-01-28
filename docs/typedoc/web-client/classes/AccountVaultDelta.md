[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / AccountVaultDelta

# Class: AccountVaultDelta

`AccountVaultDelta` stores the difference between the initial and final account vault states.

The difference is represented as follows:
- `fungible`: a binary tree map of fungible asset balance changes in the account vault.
- `non_fungible`: a binary tree map of non-fungible assets that were added to or removed from
  the account vault.

## Methods

### \[dispose\]()

> **\[dispose\]**(): `void`

#### Returns

`void`

***

### addedFungibleAssets()

> **addedFungibleAssets**(): [`FungibleAsset`](FungibleAsset.md)[]

Returns the fungible assets that increased.

#### Returns

[`FungibleAsset`](FungibleAsset.md)[]

***

### free()

> **free**(): `void`

#### Returns

`void`

***

### fungible()

> **fungible**(): [`FungibleAssetDelta`](FungibleAssetDelta.md)

Returns the fungible portion of the delta.

#### Returns

[`FungibleAssetDelta`](FungibleAssetDelta.md)

***

### isEmpty()

> **isEmpty**(): `boolean`

Returns true if no assets are changed.

#### Returns

`boolean`

***

### removedFungibleAssets()

> **removedFungibleAssets**(): [`FungibleAsset`](FungibleAsset.md)[]

Returns the fungible assets that decreased.

#### Returns

[`FungibleAsset`](FungibleAsset.md)[]

***

### serialize()

> **serialize**(): `Uint8Array`

Serializes the vault delta into bytes.

#### Returns

`Uint8Array`

***

### deserialize()

> `static` **deserialize**(`bytes`): `AccountVaultDelta`

Deserializes a vault delta from bytes.

#### Parameters

##### bytes

`Uint8Array`

#### Returns

`AccountVaultDelta`
