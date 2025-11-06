[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / ConsumableNoteRecordArray

# Class: ConsumableNoteRecordArray

## Constructors

### Constructor

> **new ConsumableNoteRecordArray**(`elements?`): `ConsumableNoteRecordArray`

#### Parameters

##### elements?

[`ConsumableNoteRecord`](ConsumableNoteRecord.md)[]

#### Returns

`ConsumableNoteRecordArray`

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

> **get**(`index`): [`ConsumableNoteRecord`](ConsumableNoteRecord.md)

Get element at index, will always return a clone to avoid aliasing issues.

#### Parameters

##### index

`number`

#### Returns

[`ConsumableNoteRecord`](ConsumableNoteRecord.md)

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

[`ConsumableNoteRecord`](ConsumableNoteRecord.md)

#### Returns

`void`

***

### replaceAt()

> **replaceAt**(`index`, `elem`): `void`

#### Parameters

##### index

`number`

##### elem

[`ConsumableNoteRecord`](ConsumableNoteRecord.md)

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
