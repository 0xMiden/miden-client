[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / TransactionRequestArray

# Class: TransactionRequestArray

## Constructors

### Constructor

> **new TransactionRequestArray**(`elements?`): `TransactionRequestArray`

#### Parameters

##### elements?

[`TransactionRequest`](TransactionRequest.md)[]

#### Returns

`TransactionRequestArray`

## Methods

### free()

> **free**(): `void`

#### Returns

`void`

***

### get()

> **get**(`index`): [`TransactionRequest`](TransactionRequest.md)

Get element at index, will always return a clone to avoid aliasing issues.

#### Parameters

##### index

`number`

#### Returns

[`TransactionRequest`](TransactionRequest.md)

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

[`TransactionRequest`](TransactionRequest.md)

#### Returns

`void`

***

### replaceAt()

> **replaceAt**(`index`, `elem`): `void`

#### Parameters

##### index

`number`

##### elem

[`TransactionRequest`](TransactionRequest.md)

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
