[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / NoteIdArray

# Class: NoteIdArray

## Constructors

### Constructor

> **new NoteIdArray**(`elements?`): `NoteIdArray`

#### Parameters

##### elements?

[`NoteId`](NoteId.md)[]

#### Returns

`NoteIdArray`

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

> **get**(`index`): [`NoteId`](NoteId.md)

Get element at index, will always return a clone to avoid aliasing issues.

#### Parameters

##### index

`number`

#### Returns

[`NoteId`](NoteId.md)

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

[`NoteId`](NoteId.md)

#### Returns

`void`

***

### replaceAt()

> **replaceAt**(`index`, `elem`): `void`

#### Parameters

##### index

`number`

##### elem

[`NoteId`](NoteId.md)

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
