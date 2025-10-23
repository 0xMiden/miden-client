[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / ExecutedTransactionArray

# Class: ExecutedTransactionArray

## Constructors

### Constructor

> **new ExecutedTransactionArray**(`elements?`): `ExecutedTransactionArray`

#### Parameters

##### elements?

[`ExecutedTransaction`](ExecutedTransaction.md)[]

#### Returns

`ExecutedTransactionArray`

## Methods

### free()

> **free**(): `void`

#### Returns

`void`

***

### get()

> **get**(`index`): [`ExecutedTransaction`](ExecutedTransaction.md)

Get element at index, will always return a clone to avoid aliasing issues.

#### Parameters

##### index

`number`

#### Returns

[`ExecutedTransaction`](ExecutedTransaction.md)

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

[`ExecutedTransaction`](ExecutedTransaction.md)

#### Returns

`void`

***

### replaceAt()

> **replaceAt**(`index`, `elem`): `void`

#### Parameters

##### index

`number`

##### elem

[`ExecutedTransaction`](ExecutedTransaction.md)

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
