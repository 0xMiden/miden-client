[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / NoteInputs

# Class: NoteInputs

A container for note inputs.

A note can be associated with up to 128 input values. Each value is represented by a single
field element. Thus, note input values can contain up to ~1 KB of data.

All inputs associated with a note can be reduced to a single commitment which is computed by
first padding the inputs with ZEROs to the next multiple of 8, and then by computing a
sequential hash of the resulting elements.

## Constructors

### Constructor

> **new NoteInputs**(`felt_array`): `NoteInputs`

Creates note inputs from a list of field elements.

#### Parameters

##### felt\_array

[`FeltArray`](FeltArray.md)

#### Returns

`NoteInputs`

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

### values()

> **values**(): [`Felt`](Felt.md)[]

Returns the raw inputs as an array of field elements.

#### Returns

[`Felt`](Felt.md)[]
