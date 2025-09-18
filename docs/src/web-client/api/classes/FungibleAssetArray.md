[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / FungibleAssetArray

# Class: FungibleAssetArray

## Constructors

### Constructor

> **new FungibleAssetArray**(`elements`?): `FungibleAssetArray`

#### Parameters

##### elements?

[`FungibleAsset`](FungibleAsset.md)[]

#### Returns

`FungibleAssetArray`

## Methods

### free()

> **free**(): `void`

#### Returns

`void`

***

### get()

> **get**(`index`): [`FungibleAsset`](FungibleAsset.md)

Get element at index, will always return a clone to avoid aliasing issues.

#### Parameters

##### index

`number`

#### Returns

[`FungibleAsset`](FungibleAsset.md)

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

[`FungibleAsset`](FungibleAsset.md)

#### Returns

`void`

***

### replaceAt()

> **replaceAt**(`index`, `elem`): `void`

#### Parameters

##### index

`number`

##### elem

[`FungibleAsset`](FungibleAsset.md)

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
