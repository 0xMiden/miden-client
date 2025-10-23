[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / AdviceMapArray

# Class: AdviceMapArray

## Constructors

### Constructor

> **new AdviceMapArray**(`elements?`): `AdviceMapArray`

#### Parameters

##### elements?

[`AdviceMap`](AdviceMap.md)[]

#### Returns

`AdviceMapArray`

## Methods

### free()

> **free**(): `void`

#### Returns

`void`

***

### get()

> **get**(`index`): [`AdviceMap`](AdviceMap.md)

Get element at index, will always return a clone to avoid aliasing issues.

#### Parameters

##### index

`number`

#### Returns

[`AdviceMap`](AdviceMap.md)

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

[`AdviceMap`](AdviceMap.md)

#### Returns

`void`

***

### replaceAt()

> **replaceAt**(`index`, `elem`): `void`

#### Parameters

##### index

`number`

##### elem

[`AdviceMap`](AdviceMap.md)

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
