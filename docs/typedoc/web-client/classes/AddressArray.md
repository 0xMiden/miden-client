[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / AddressArray

# Class: AddressArray

## Constructors

### Constructor

> **new AddressArray**(`elements?`): `AddressArray`

#### Parameters

##### elements?

[`Address`](Address.md)[]

#### Returns

`AddressArray`

## Methods

### free()

> **free**(): `void`

#### Returns

`void`

***

### get()

> **get**(`index`): [`Address`](Address.md)

Get element at index, will always return a clone to avoid aliasing issues.

#### Parameters

##### index

`number`

#### Returns

[`Address`](Address.md)

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

[`Address`](Address.md)

#### Returns

`void`

***

### replaceAt()

> **replaceAt**(`index`, `elem`): `void`

#### Parameters

##### index

`number`

##### elem

[`Address`](Address.md)

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
