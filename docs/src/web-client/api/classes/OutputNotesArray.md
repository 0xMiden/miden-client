[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / OutputNotesArray

# Class: OutputNotesArray

## Constructors

### Constructor

> **new OutputNotesArray**(`elements?`): `OutputNotesArray`

#### Parameters

##### elements?

[`OutputNotes`](OutputNotes.md)[]

#### Returns

`OutputNotesArray`

## Methods

### free()

> **free**(): `void`

#### Returns

`void`

***

### get()

> **get**(`index`): [`OutputNotes`](OutputNotes.md)

Get element at index, will always return a clone to avoid aliasing issues.

#### Parameters

##### index

`number`

#### Returns

[`OutputNotes`](OutputNotes.md)

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

[`OutputNotes`](OutputNotes.md)

#### Returns

`void`

***

### replaceAt()

> **replaceAt**(`index`, `elem`): `void`

#### Parameters

##### index

`number`

##### elem

[`OutputNotes`](OutputNotes.md)

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
