[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / AccountStorageRequirementsArray

# Class: AccountStorageRequirementsArray

## Constructors

### Constructor

> **new AccountStorageRequirementsArray**(`elements`?): `AccountStorageRequirementsArray`

#### Parameters

##### elements?

[`AccountStorageRequirements`](AccountStorageRequirements.md)[]

#### Returns

`AccountStorageRequirementsArray`

## Methods

### free()

> **free**(): `void`

#### Returns

`void`

***

### get()

> **get**(`index`): [`AccountStorageRequirements`](AccountStorageRequirements.md)

Get element at index, will always return a clone to avoid aliasing issues.

#### Parameters

##### index

`number`

#### Returns

[`AccountStorageRequirements`](AccountStorageRequirements.md)

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

[`AccountStorageRequirements`](AccountStorageRequirements.md)

#### Returns

`void`

***

### replaceAt()

> **replaceAt**(`index`, `elem`): `void`

#### Parameters

##### index

`number`

##### elem

[`AccountStorageRequirements`](AccountStorageRequirements.md)

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
