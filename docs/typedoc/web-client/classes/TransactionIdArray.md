[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / TransactionIdArray

# Class: TransactionIdArray

## Constructors

### Constructor

> **new TransactionIdArray**(`elements?`): `TransactionIdArray`

#### Parameters

##### elements?

[`TransactionId`](TransactionId.md)[]

#### Returns

`TransactionIdArray`

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

### get()

> **get**(`index`): [`TransactionId`](TransactionId.md)

Get element at index, will always return a clone to avoid aliasing issues.

#### Parameters

##### index

`number`

#### Returns

[`TransactionId`](TransactionId.md)

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

[`TransactionId`](TransactionId.md)

#### Returns

`void`

***

### replaceAt()

> **replaceAt**(`index`, `elem`): `void`

#### Parameters

##### index

`number`

##### elem

[`TransactionId`](TransactionId.md)

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
