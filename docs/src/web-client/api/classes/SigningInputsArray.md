[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / SigningInputsArray

# Class: SigningInputsArray

## Constructors

### Constructor

> **new SigningInputsArray**(`elements?`): `SigningInputsArray`

#### Parameters

##### elements?

[`SigningInputs`](SigningInputs.md)[]

#### Returns

`SigningInputsArray`

## Methods

### free()

> **free**(): `void`

#### Returns

`void`

***

### get()

> **get**(`index`): [`SigningInputs`](SigningInputs.md)

Get element at index, will always return a clone to avoid aliasing issues.

#### Parameters

##### index

`number`

#### Returns

[`SigningInputs`](SigningInputs.md)

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

[`SigningInputs`](SigningInputs.md)

#### Returns

`void`

***

### replaceAt()

> **replaceAt**(`index`, `elem`): `void`

#### Parameters

##### index

`number`

##### elem

[`SigningInputs`](SigningInputs.md)

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
