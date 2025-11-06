[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / NoteFilterArray

# Class: NoteFilterArray

## Constructors

### Constructor

> **new NoteFilterArray**(`elements?`): `NoteFilterArray`

#### Parameters

##### elements?

[`NoteFilter`](NoteFilter.md)[]

#### Returns

`NoteFilterArray`

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

> **get**(`index`): [`NoteFilter`](NoteFilter.md)

Get element at index, will always return a clone to avoid aliasing issues.

#### Parameters

##### index

`number`

#### Returns

[`NoteFilter`](NoteFilter.md)

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

[`NoteFilter`](NoteFilter.md)

#### Returns

`void`

***

### replaceAt()

> **replaceAt**(`index`, `elem`): `void`

#### Parameters

##### index

`number`

##### elem

[`NoteFilter`](NoteFilter.md)

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
