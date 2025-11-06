[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / AccountArray

# Class: AccountArray

## Constructors

### Constructor

> **new AccountArray**(`elements?`): `AccountArray`

#### Parameters

##### elements?

[`Account`](Account.md)[]

#### Returns

`AccountArray`

## Methods

### free()

> **free**(): `void`

#### Returns

`void`

***

### get()

> **get**(`index`): [`Account`](Account.md)

Get element at index, will always return a clone to avoid aliasing issues.

#### Parameters

##### index

`number`

#### Returns

[`Account`](Account.md)

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

[`Account`](Account.md)

#### Returns

`void`

***

### replaceAt()

> **replaceAt**(`index`, `elem`): `void`

#### Parameters

##### index

`number`

##### elem

[`Account`](Account.md)

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
