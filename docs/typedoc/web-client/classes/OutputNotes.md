[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / OutputNotes

# Class: OutputNotes

## Methods

### \[dispose\]()

> **\[dispose\]**(): `void`

#### Returns

`void`

***

### commitment()

> **commitment**(): [`Word`](Word.md)

Returns the commitment to all output notes.

#### Returns

[`Word`](Word.md)

***

### free()

> **free**(): `void`

#### Returns

`void`

***

### getNote()

> **getNote**(`index`): [`OutputNote`](OutputNote.md)

Returns the output note at the specified index.

#### Parameters

##### index

`number`

#### Returns

[`OutputNote`](OutputNote.md)

***

### isEmpty()

> **isEmpty**(): `boolean`

Returns true if there are no output notes.

#### Returns

`boolean`

***

### notes()

> **notes**(): [`OutputNote`](OutputNote.md)[]

Returns all output notes as a vector.

#### Returns

[`OutputNote`](OutputNote.md)[]

***

### numNotes()

> **numNotes**(): `number`

Returns the number of notes emitted.

#### Returns

`number`
