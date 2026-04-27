[**@miden-sdk/miden-wallet-adapter-base**](../README.md)

***

[@miden-sdk/miden-wallet-adapter-base](../README.md) / WalletAdapterEvents

# Interface: WalletAdapterEvents

## Methods

### connect()

> **connect**(`address`, `allowedPrivateData?`): `void`

#### Parameters

##### address

`string`

##### allowedPrivateData?

[`AllowedPrivateData`](../enumerations/AllowedPrivateData.md)

#### Returns

`void`

***

### disconnect()

> **disconnect**(): `void`

#### Returns

`void`

***

### error()

> **error**(`error`): `void`

#### Parameters

##### error

[`WalletError`](../classes/WalletError.md)

#### Returns

`void`

***

### readyStateChange()

> **readyStateChange**(`readyState`): `void`

#### Parameters

##### readyState

[`WalletReadyState`](../enumerations/WalletReadyState.md)

#### Returns

`void`
