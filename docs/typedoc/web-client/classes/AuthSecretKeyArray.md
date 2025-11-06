[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / AuthSecretKeyArray

# Class: AuthSecretKeyArray

## Constructors

### Constructor

> **new AuthSecretKeyArray**(`elements?`): `AuthSecretKeyArray`

#### Parameters

##### elements?

[`AuthSecretKey`](AuthSecretKey.md)[]

#### Returns

`AuthSecretKeyArray`

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

> **get**(`index`): [`AuthSecretKey`](AuthSecretKey.md)

Get element at index, will always return a clone to avoid aliasing issues.

#### Parameters

##### index

`number`

#### Returns

[`AuthSecretKey`](AuthSecretKey.md)

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

[`AuthSecretKey`](AuthSecretKey.md)

#### Returns

`void`

***

### replaceAt()

> **replaceAt**(`index`, `elem`): `void`

#### Parameters

##### index

`number`

##### elem

[`AuthSecretKey`](AuthSecretKey.md)

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
