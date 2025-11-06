[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / TransactionSummaryArray

# Class: TransactionSummaryArray

## Constructors

### Constructor

> **new TransactionSummaryArray**(`elements?`): `TransactionSummaryArray`

#### Parameters

##### elements?

[`TransactionSummary`](TransactionSummary.md)[]

#### Returns

`TransactionSummaryArray`

## Methods

### free()

> **free**(): `void`

#### Returns

`void`

***

### get()

> **get**(`index`): [`TransactionSummary`](TransactionSummary.md)

Get element at index, will always return a clone to avoid aliasing issues.

#### Parameters

##### index

`number`

#### Returns

[`TransactionSummary`](TransactionSummary.md)

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

[`TransactionSummary`](TransactionSummary.md)

#### Returns

`void`

***

### replaceAt()

> **replaceAt**(`index`, `elem`): `void`

#### Parameters

##### index

`number`

##### elem

[`TransactionSummary`](TransactionSummary.md)

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
