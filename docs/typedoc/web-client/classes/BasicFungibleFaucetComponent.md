[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / BasicFungibleFaucetComponent

# Class: BasicFungibleFaucetComponent

View over a basic fungible faucet account component.

## Methods

### decimals()

> **decimals**(): `number`

Returns the number of decimals used by the faucet token.

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

Returns the maximum supply the faucet is allowed to mint.

#### Returns

[`Felt`](Felt.md)

***

### symbol()

> **symbol**(): [`TokenSymbol`](TokenSymbol.md)

Returns the faucet token symbol.

#### Returns

[`TokenSymbol`](TokenSymbol.md)

***

### fromAccount()

> `static` **fromAccount**(`account`): `BasicFungibleFaucetComponent`

Extracts faucet metadata from an existing account.

#### Parameters

##### account

[`Account`](Account.md)

#### Returns

`BasicFungibleFaucetComponent`

#### Throws

Throws if the account is not a basic fungible faucet.
