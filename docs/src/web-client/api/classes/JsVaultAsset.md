---
title: JsVaultAsset
draft: true
---

[**@demox-labs/miden-sdk**](../index)

***

[@demox-labs/miden-sdk](../index) / JsVaultAsset

# Class: JsVaultAsset

An object that contains a serialized vault asset

## Properties

### asset

> **asset**: `string`

Word representing the asset.

***

### faucetIdPrefix

> **faucetIdPrefix**: `string`

Asset's faucet ID prefix.

***

### root

> **root**: `string`

The merkle root of the vault's assets.

***

### vaultKey

> **vaultKey**: `string`

The vault key associated with the asset.

## Methods

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
