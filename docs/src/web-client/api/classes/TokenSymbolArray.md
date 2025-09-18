[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / TokenSymbolArray

# Class: TokenSymbolArray

## Constructors

### Constructor

> **new TokenSymbolArray**(`elements`?): `TokenSymbolArray`

#### Parameters

##### elements?

[`TokenSymbol`](TokenSymbol.md)[]

#### Returns

`TokenSymbolArray`

## Methods

### free()

> **free**(): `void`

#### Returns

`void`

***

### get()

> **get**(`index`): [`TokenSymbol`](TokenSymbol.md)

Get element at index, will always return a clone to avoid aliasing issues.

#### Parameters

##### index

`number`

#### Returns

[`TokenSymbol`](TokenSymbol.md)

***

### length()

> **length**(): `number`

#### Returns

`number`

***

### push()

> **push**(`element`): `void`

#### Parameters

##### element

[`TokenSymbol`](TokenSymbol.md)

#### Returns

`void`

***

### replaceAt()

> **replaceAt**(`index`, `elem`): `void`

#### Parameters

##### index

`number`

##### elem

[`TokenSymbol`](TokenSymbol.md)

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
