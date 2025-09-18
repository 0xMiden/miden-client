[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / NoteTagArray

# Class: NoteTagArray

## Constructors

### Constructor

> **new NoteTagArray**(`elements`?): `NoteTagArray`

#### Parameters

##### elements?

[`NoteTag`](NoteTag.md)[]

#### Returns

`NoteTagArray`

## Methods

### free()

> **free**(): `void`

#### Returns

`void`

***

### get()

> **get**(`index`): [`NoteTag`](NoteTag.md)

Get element at index, will always return a clone to avoid aliasing issues.

#### Parameters

##### index

`number`

#### Returns

[`NoteTag`](NoteTag.md)

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

[`NoteTag`](NoteTag.md)

#### Returns

`void`

***

### replaceAt()

> **replaceAt**(`index`, `elem`): `void`

#### Parameters

##### index

`number`

##### elem

[`NoteTag`](NoteTag.md)

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
