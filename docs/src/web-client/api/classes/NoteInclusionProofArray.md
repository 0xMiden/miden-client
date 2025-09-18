[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / NoteInclusionProofArray

# Class: NoteInclusionProofArray

## Constructors

### Constructor

> **new NoteInclusionProofArray**(`elements`?): `NoteInclusionProofArray`

#### Parameters

##### elements?

[`NoteInclusionProof`](NoteInclusionProof.md)[]

#### Returns

`NoteInclusionProofArray`

## Methods

### free()

> **free**(): `void`

#### Returns

`void`

***

### get()

> **get**(`index`): [`NoteInclusionProof`](NoteInclusionProof.md)

Get element at index, will always return a clone to avoid aliasing issues.

#### Parameters

##### index

`number`

#### Returns

[`NoteInclusionProof`](NoteInclusionProof.md)

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

[`NoteInclusionProof`](NoteInclusionProof.md)

#### Returns

`void`

***

### replaceAt()

> **replaceAt**(`index`, `elem`): `void`

#### Parameters

##### index

`number`

##### elem

[`NoteInclusionProof`](NoteInclusionProof.md)

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
