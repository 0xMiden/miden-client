[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / NoteIdAndArgsArray

# Class: NoteIdAndArgsArray

## Constructors

### Constructor

> **new NoteIdAndArgsArray**(`elements`?): `NoteIdAndArgsArray`

#### Parameters

##### elements?

[`NoteIdAndArgs`](NoteIdAndArgs.md)[]

#### Returns

`NoteIdAndArgsArray`

## Methods

### free()

> **free**(): `void`

#### Returns

`void`

***

### get()

> **get**(`index`): [`NoteIdAndArgs`](NoteIdAndArgs.md)

Get element at index, will always return a clone to avoid aliasing issues.

#### Parameters

##### index

`number`

#### Returns

[`NoteIdAndArgs`](NoteIdAndArgs.md)

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

[`NoteIdAndArgs`](NoteIdAndArgs.md)

#### Returns

`void`

***

### replaceAt()

> **replaceAt**(`index`, `elem`): `void`

#### Parameters

##### index

`number`

##### elem

[`NoteIdAndArgs`](NoteIdAndArgs.md)

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
