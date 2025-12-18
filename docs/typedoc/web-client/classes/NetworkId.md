[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / NetworkId

# Class: NetworkId

The identifier of a Miden network.

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

### custom()

> `static` **custom**(`custom_prefix`): `NetworkId`

Builds a custom network ID from a provided custom prefix.

Returns an error if the prefix is invalid.

#### Parameters

##### custom\_prefix

`string`

#### Returns

`NetworkId`

***

### devnet()

> `static` **devnet**(): `NetworkId`

#### Returns

`NetworkId`

***

### mainnet()

> `static` **mainnet**(): `NetworkId`

#### Returns

`NetworkId`

***

### testnet()

> `static` **testnet**(): `NetworkId`

#### Returns

`NetworkId`
