[**@miden-sdk/miden-wallet-adapter-base**](../README.md)

***

[@miden-sdk/miden-wallet-adapter-base](../README.md) / SignerWalletAdapterProps

# Interface: SignerWalletAdapterProps\<Name\>

## Extends

- [`WalletAdapterProps`](WalletAdapterProps.md)\<`Name`\>

## Type Parameters

### Name

`Name` *extends* `string` = `string`

## Properties

### address

> **address**: `string`

#### Inherited from

[`WalletAdapterProps`](WalletAdapterProps.md).[`address`](WalletAdapterProps.md#address)

***

### connected

> **connected**: `boolean`

#### Inherited from

[`WalletAdapterProps`](WalletAdapterProps.md).[`connected`](WalletAdapterProps.md#connected)

***

### connecting

> **connecting**: `boolean`

#### Inherited from

[`WalletAdapterProps`](WalletAdapterProps.md).[`connecting`](WalletAdapterProps.md#connecting)

***

### icon

> **icon**: `string`

#### Inherited from

[`WalletAdapterProps`](WalletAdapterProps.md).[`icon`](WalletAdapterProps.md#icon)

***

### name

> **name**: [`WalletName`](../type-aliases/WalletName.md)\<`Name`\>

#### Inherited from

[`WalletAdapterProps`](WalletAdapterProps.md).[`name`](WalletAdapterProps.md#name-1)

***

### publicKey

> **publicKey**: `Uint8Array`\<`ArrayBufferLike`\>

#### Inherited from

[`WalletAdapterProps`](WalletAdapterProps.md).[`publicKey`](WalletAdapterProps.md#publickey)

***

### readyState

> **readyState**: [`WalletReadyState`](../enumerations/WalletReadyState.md)

#### Inherited from

[`WalletAdapterProps`](WalletAdapterProps.md).[`readyState`](WalletAdapterProps.md#readystate)

***

### supportedTransactionVersions

> **supportedTransactionVersions**: `ReadonlySet`

#### Inherited from

[`WalletAdapterProps`](WalletAdapterProps.md).[`supportedTransactionVersions`](WalletAdapterProps.md#supportedtransactionversions)

***

### url

> **url**: `string`

#### Inherited from

[`WalletAdapterProps`](WalletAdapterProps.md).[`url`](WalletAdapterProps.md#url)

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

#### Inherited from

[`WalletAdapterProps`](WalletAdapterProps.md).[`connect`](WalletAdapterProps.md#connect)

***

### disconnect()

> **disconnect**(): `Promise`\<`void`\>

#### Returns

`Promise`\<`void`\>

#### Inherited from

[`WalletAdapterProps`](WalletAdapterProps.md).[`disconnect`](WalletAdapterProps.md#disconnect)
