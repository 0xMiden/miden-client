[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / AccountVaultDelta

# Class: AccountVaultDelta

Wrapper around [`miden_client::asset::AccountVaultDelta`].

Describes changes to the fungible assets stored in an account's vault.

## Methods

### addedFungibleAssets()

> **addedFungibleAssets**(): [`FungibleAsset`](FungibleAsset.md)[]

Returns fungible assets that were added, paired with the increase amount.

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

Returns the aggregated fungible asset delta for this vault.

#### Returns

[`FungibleAssetDelta`](FungibleAssetDelta.md)

***

### isEmpty()

> **isEmpty**(): `boolean`

Returns `true` if the vault delta leaves all balances unchanged.

#### Returns

`boolean`

***

### removedFungibleAssets()

> **removedFungibleAssets**(): [`FungibleAsset`](FungibleAsset.md)[]

Returns fungible assets that were removed, paired with the decrease amount.

#### Returns

[`FungibleAsset`](FungibleAsset.md)[]

***

### serialize()

> **serialize**(): `Uint8Array`

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

#### Throws

Throws if the byte representation is invalid.
