[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / BasicFungibleFaucetComponent

# Class: BasicFungibleFaucetComponent

Provides metadata for a basic fungible faucet account component.

## Methods

### \[dispose\]()

> **\[dispose\]**(): `void`

#### Returns

`void`

***

### decimals()

> **decimals**(): `number`

Returns the number of decimal places for the token.

#### Returns

`number`

***

### free()

> **free**(): `void`

#### Returns

`void`

***

### maxSupply()

> **maxSupply**(): [`Felt`](Felt.md)

Returns the maximum token supply.

#### Returns

[`Felt`](Felt.md)

***

### symbol()

> **symbol**(): [`TokenSymbol`](TokenSymbol.md)

Returns the faucet's token symbol.

#### Returns

[`TokenSymbol`](TokenSymbol.md)

***

### fromAccount()

> `static` **fromAccount**(`account`): `BasicFungibleFaucetComponent`

Extracts faucet metadata from an account.

#### Parameters

##### account

[`Account`](Account.md)

#### Returns

`BasicFungibleFaucetComponent`
