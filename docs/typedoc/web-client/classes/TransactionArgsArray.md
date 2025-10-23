[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / TransactionArgsArray

# Class: TransactionArgsArray

## Constructors

### Constructor

> **new TransactionArgsArray**(`elements?`): `TransactionArgsArray`

#### Parameters

##### elements?

[`TransactionArgs`](TransactionArgs.md)[]

#### Returns

`TransactionArgsArray`

## Methods

### free()

> **free**(): `void`

#### Returns

`void`

***

### get()

> **get**(`index`): [`TransactionArgs`](TransactionArgs.md)

Get element at index, will always return a clone to avoid aliasing issues.

#### Parameters

##### index

`number`

#### Returns

[`TransactionArgs`](TransactionArgs.md)

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

[`TransactionArgs`](TransactionArgs.md)

#### Returns

`void`

***

### replaceAt()

> **replaceAt**(`index`, `elem`): `void`

#### Parameters

##### index

`number`

##### elem

[`TransactionArgs`](TransactionArgs.md)

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
