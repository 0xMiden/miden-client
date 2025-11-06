[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / SecretKeyArray

# Class: SecretKeyArray

## Constructors

### Constructor

> **new SecretKeyArray**(`elements?`): `SecretKeyArray`

#### Parameters

##### elements?

[`SecretKey`](SecretKey.md)[]

#### Returns

`SecretKeyArray`

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

> **get**(`index`): [`SecretKey`](SecretKey.md)

Get element at index, will always return a clone to avoid aliasing issues.

#### Parameters

##### index

`number`

#### Returns

[`SecretKey`](SecretKey.md)

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

[`SecretKey`](SecretKey.md)

#### Returns

`void`

***

### replaceAt()

> **replaceAt**(`index`, `elem`): `void`

#### Parameters

##### index

`number`

##### elem

[`SecretKey`](SecretKey.md)

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
