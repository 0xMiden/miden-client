[**@miden-sdk/miden-wallet-adapter-react**](../README.md)

***

[@miden-sdk/miden-wallet-adapter-react](../README.md) / MidenFiSignerProviderProps

# Interface: MidenFiSignerProviderProps

## Properties

### allowedPrivateData?

> `optional` **allowedPrivateData**: `AllowedPrivateData`

Allowed private data types

***

### appName?

> `optional` **appName**: `string`

App name passed to the default MidenWalletAdapter

***

### autoConnect?

> `optional` **autoConnect**: `boolean`

Auto-connect to previously selected wallet on mount. Defaults to true

***

### children

> **children**: `ReactNode`

***

### localStorageKey?

> `optional` **localStorageKey**: `string`

LocalStorage key for persisting wallet selection

***

### network?

> `optional` **network**: `WalletAdapterNetwork`

Network to connect to

***

### onError()?

> `optional` **onError**: (`error`) => `void`

Error handler

#### Parameters

##### error

`WalletError`

#### Returns

`void`

***

### privateDataPermission?

> `optional` **privateDataPermission**: `PrivateDataPermission`

Private data permission level

***

### wallets?

> `optional` **wallets**: `Adapter`[]

Wallet adapters to use. Defaults to [MidenWalletAdapter]
