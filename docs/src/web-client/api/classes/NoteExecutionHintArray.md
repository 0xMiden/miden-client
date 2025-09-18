[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / NoteExecutionHintArray

# Class: NoteExecutionHintArray

## Constructors

### Constructor

> **new NoteExecutionHintArray**(`elements`?): `NoteExecutionHintArray`

#### Parameters

##### elements?

[`NoteExecutionHint`](NoteExecutionHint.md)[]

#### Returns

`NoteExecutionHintArray`

## Methods

### free()

> **free**(): `void`

#### Returns

`void`

***

### get()

> **get**(`index`): [`NoteExecutionHint`](NoteExecutionHint.md)

Get element at index, will always return a clone to avoid aliasing issues.

#### Parameters

##### index

`number`

#### Returns

[`NoteExecutionHint`](NoteExecutionHint.md)

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

[`NoteExecutionHint`](NoteExecutionHint.md)

#### Returns

`void`

***

### replaceAt()

> **replaceAt**(`index`, `elem`): `void`

#### Parameters

##### index

`number`

##### elem

[`NoteExecutionHint`](NoteExecutionHint.md)

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
