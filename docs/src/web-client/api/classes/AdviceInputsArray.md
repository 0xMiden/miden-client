[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / AdviceInputsArray

# Class: AdviceInputsArray

## Constructors

### Constructor

> **new AdviceInputsArray**(`elements`?): `AdviceInputsArray`

#### Parameters

##### elements?

[`AdviceInputs`](AdviceInputs.md)[]

#### Returns

`AdviceInputsArray`

## Methods

### free()

> **free**(): `void`

#### Returns

`void`

***

### get()

> **get**(`index`): [`AdviceInputs`](AdviceInputs.md)

Get element at index, will always return a clone to avoid aliasing issues.

#### Parameters

##### index

`number`

#### Returns

[`AdviceInputs`](AdviceInputs.md)

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

[`AdviceInputs`](AdviceInputs.md)

#### Returns

`void`

***

### replaceAt()

> **replaceAt**(`index`, `elem`): `void`

#### Parameters

##### index

`number`

##### elem

[`AdviceInputs`](AdviceInputs.md)

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
