[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / JsVaultAsset

# Class: JsVaultAsset

An object that contains a serialized vault asset

## Properties

### accountId

> **accountId**: `string`

The account ID this asset belongs to.

***

### asset

> **asset**: `string`

Word representing the asset.

***

### faucetIdPrefix

> **faucetIdPrefix**: `string`

Asset's faucet ID prefix.

***

### nonce

> **nonce**: `string`

The account's nonce when this asset state was recorded.

***

### vaultKey

> **vaultKey**: `string`

The vault key associated with the asset.

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

### toJSON()

> **toJSON**(): `Object`

* Return copy of self without private attributes.

#### Returns

`Object`

***

### toString()

> **toString**(): `string`

Return stringified version of self.

#### Returns

`string`
