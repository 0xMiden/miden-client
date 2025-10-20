[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / NoteAndArgsArray

# Class: NoteAndArgsArray

## Constructors

### Constructor

> **new NoteAndArgsArray**(`elements?`): `NoteAndArgsArray`

#### Parameters

##### elements?

[`NoteAndArgs`](NoteAndArgs.md)[]

#### Returns

`NoteAndArgsArray`

## Methods

### free()

> **free**(): `void`

#### Returns

`void`

***

### get()

> **get**(`index`): [`NoteAndArgs`](NoteAndArgs.md)

Get element at index, will always return a clone to avoid aliasing issues.

#### Parameters

##### index

`number`

#### Returns

[`NoteAndArgs`](NoteAndArgs.md)

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

[`NoteAndArgs`](NoteAndArgs.md)

#### Returns

`void`

***

### replaceAt()

> **replaceAt**(`index`, `elem`): `void`

#### Parameters

##### index

`number`

##### elem

[`NoteAndArgs`](NoteAndArgs.md)

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
