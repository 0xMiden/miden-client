[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / TokenSymbol

# Class: TokenSymbol

Represents a string token symbol (e.g. "POL", "ETH") as a single [\`Felt\`](Felt.md) value.

Token Symbols can consists of up to 6 capital Latin characters, e.g. "C", "ETH", "MIDENC".

## Constructors

### Constructor

> **new TokenSymbol**(`symbol`): `TokenSymbol`

Creates a token symbol from a string.

#### Parameters

##### symbol

`string`

#### Returns

`TokenSymbol`

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

### toString()

> **toString**(): `string`

Returns the validated symbol string.

#### Returns

`string`
