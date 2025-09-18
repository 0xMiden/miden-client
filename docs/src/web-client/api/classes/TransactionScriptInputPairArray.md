[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / TransactionScriptInputPairArray

# Class: TransactionScriptInputPairArray

## Constructors

### Constructor

> **new TransactionScriptInputPairArray**(`elements`?): `TransactionScriptInputPairArray`

#### Parameters

##### elements?

[`TransactionScriptInputPair`](TransactionScriptInputPair.md)[]

#### Returns

`TransactionScriptInputPairArray`

## Methods

### free()

> **free**(): `void`

#### Returns

`void`

***

### get()

> **get**(`index`): [`TransactionScriptInputPair`](TransactionScriptInputPair.md)

Get element at index, will always return a clone to avoid aliasing issues.

#### Parameters

##### index

`number`

#### Returns

[`TransactionScriptInputPair`](TransactionScriptInputPair.md)

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

[`TransactionScriptInputPair`](TransactionScriptInputPair.md)

#### Returns

`void`

***

### replaceAt()

> **replaceAt**(`index`, `elem`): `void`

#### Parameters

##### index

`number`

##### elem

[`TransactionScriptInputPair`](TransactionScriptInputPair.md)

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
