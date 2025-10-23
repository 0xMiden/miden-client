[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / AccountHeaderArray

# Class: AccountHeaderArray

## Constructors

### Constructor

> **new AccountHeaderArray**(`elements?`): `AccountHeaderArray`

#### Parameters

##### elements?

[`AccountHeader`](AccountHeader.md)[]

#### Returns

`AccountHeaderArray`

## Methods

### free()

> **free**(): `void`

#### Returns

`void`

***

### get()

> **get**(`index`): [`AccountHeader`](AccountHeader.md)

Get element at index, will always return a clone to avoid aliasing issues.

#### Parameters

##### index

`number`

#### Returns

[`AccountHeader`](AccountHeader.md)

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

[`AccountHeader`](AccountHeader.md)

#### Returns

`void`

***

### replaceAt()

> **replaceAt**(`index`, `elem`): `void`

#### Parameters

##### index

`number`

##### elem

[`AccountHeader`](AccountHeader.md)

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
