[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / InputNotes

# Class: InputNotes

## Methods

### \[dispose\]()

> **\[dispose\]**(): `void`

#### Returns

`void`

***

### commitment()

> **commitment**(): [`Word`](Word.md)

Returns the commitment to all input notes.

#### Returns

[`Word`](Word.md)

***

### free()

> **free**(): `void`

#### Returns

`void`

***

### getNote()

> **getNote**(`index`): [`InputNote`](InputNote.md)

Returns the input note at the specified index.

#### Parameters

##### index

`number`

#### Returns

[`InputNote`](InputNote.md)

***

### isEmpty()

> **isEmpty**(): `boolean`

Returns true if there are no input notes.

#### Returns

`boolean`

***

### notes()

> **notes**(): [`InputNote`](InputNote.md)[]

Returns all input notes as a vector.

#### Returns

[`InputNote`](InputNote.md)[]

***

### numNotes()

> **numNotes**(): `number`

Returns the number of input notes.

#### Returns

`number`
