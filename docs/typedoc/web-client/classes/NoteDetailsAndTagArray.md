[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / NoteDetailsAndTagArray

# Class: NoteDetailsAndTagArray

## Constructors

### Constructor

> **new NoteDetailsAndTagArray**(`elements?`): `NoteDetailsAndTagArray`

#### Parameters

##### elements?

[`NoteDetailsAndTag`](NoteDetailsAndTag.md)[]

#### Returns

`NoteDetailsAndTagArray`

## Methods

### free()

> **free**(): `void`

#### Returns

`void`

***

### get()

> **get**(`index`): [`NoteDetailsAndTag`](NoteDetailsAndTag.md)

Get element at index, will always return a clone to avoid aliasing issues.

#### Parameters

##### index

`number`

#### Returns

[`NoteDetailsAndTag`](NoteDetailsAndTag.md)

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

[`NoteDetailsAndTag`](NoteDetailsAndTag.md)

#### Returns

`void`

***

### replaceAt()

> **replaceAt**(`index`, `elem`): `void`

#### Parameters

##### index

`number`

##### elem

[`NoteDetailsAndTag`](NoteDetailsAndTag.md)

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
