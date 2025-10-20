[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / ForeignAccountArray

# Class: ForeignAccountArray

## Constructors

### Constructor

> **new ForeignAccountArray**(`elements?`): `ForeignAccountArray`

#### Parameters

##### elements?

[`ForeignAccount`](ForeignAccount.md)[]

#### Returns

`ForeignAccountArray`

## Methods

### free()

> **free**(): `void`

#### Returns

`void`

***

### get()

> **get**(`index`): [`ForeignAccount`](ForeignAccount.md)

Get element at index, will always return a clone to avoid aliasing issues.

#### Parameters

##### index

`number`

#### Returns

[`ForeignAccount`](ForeignAccount.md)

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

[`ForeignAccount`](ForeignAccount.md)

#### Returns

`void`

***

### replaceAt()

> **replaceAt**(`index`, `elem`): `void`

#### Parameters

##### index

`number`

##### elem

[`ForeignAccount`](ForeignAccount.md)

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
