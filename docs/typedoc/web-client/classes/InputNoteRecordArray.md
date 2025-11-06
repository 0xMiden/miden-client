[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / InputNoteRecordArray

# Class: InputNoteRecordArray

## Constructors

### Constructor

> **new InputNoteRecordArray**(`elements?`): `InputNoteRecordArray`

#### Parameters

##### elements?

[`InputNoteRecord`](InputNoteRecord.md)[]

#### Returns

`InputNoteRecordArray`

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

> **get**(`index`): [`InputNoteRecord`](InputNoteRecord.md)

Get element at index, will always return a clone to avoid aliasing issues.

#### Parameters

##### index

`number`

#### Returns

[`InputNoteRecord`](InputNoteRecord.md)

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

[`InputNoteRecord`](InputNoteRecord.md)

#### Returns

`void`

***

### replaceAt()

> **replaceAt**(`index`, `elem`): `void`

#### Parameters

##### index

`number`

##### elem

[`InputNoteRecord`](InputNoteRecord.md)

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
