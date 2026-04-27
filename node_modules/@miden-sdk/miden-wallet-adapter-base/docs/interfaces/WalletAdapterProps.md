[**@miden-sdk/miden-wallet-adapter-base**](../README.md)

***

[@miden-sdk/miden-wallet-adapter-base](../README.md) / WalletAdapterProps

# Interface: WalletAdapterProps\<Name\>

## Extended by

- [`SignerWalletAdapterProps`](SignerWalletAdapterProps.md)
- [`MessageSignerWalletAdapterProps`](MessageSignerWalletAdapterProps.md)

## Type Parameters

### Name

`Name` *extends* `string` = `string`

## Properties

### address

> **address**: `string`

***

### connected

> **connected**: `boolean`

***

### connecting

> **connecting**: `boolean`

***

### icon

> **icon**: `string`

***

### name

> **name**: [`WalletName`](../type-aliases/WalletName.md)\<`Name`\>

***

### publicKey

> **publicKey**: `Uint8Array`\<`ArrayBufferLike`\>

***

### readyState

> **readyState**: [`WalletReadyState`](../enumerations/WalletReadyState.md)

***

### supportedTransactionVersions

> **supportedTransactionVersions**: `ReadonlySet`

***

### url

> **url**: `string`

## Methods

### connect()

> **connect**(`privateDataPermission`, `network`, `allowedPrivateData?`): `Promise`\<`void`\>

#### Parameters

##### privateDataPermission

[`PrivateDataPermission`](../enumerations/PrivateDataPermission.md)

##### network

[`WalletAdapterNetwork`](../enumerations/WalletAdapterNetwork.md)

##### allowedPrivateData?

[`AllowedPrivateData`](../enumerations/AllowedPrivateData.md)

#### Returns

`Promise`\<`void`\>

***

### disconnect()

> **disconnect**(): `Promise`\<`void`\>

#### Returns

`Promise`\<`void`\>
