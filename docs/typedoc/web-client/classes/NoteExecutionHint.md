[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / NoteExecutionHint

# Class: NoteExecutionHint

Hint describing when a note can be consumed.

## Methods

### \[dispose\]()

> **\[dispose\]**(): `void`

#### Returns

`void`

***

### canBeConsumed()

> **canBeConsumed**(`block_num`): `boolean`

Returns whether the note can be consumed at the provided block height.

#### Parameters

##### block\_num

`number`

#### Returns

`boolean`

***

### free()

> **free**(): `void`

#### Returns

`void`

***

### afterBlock()

> `static` **afterBlock**(`block_num`): `NoteExecutionHint`

Creates a hint that activates after the given block number.

#### Parameters

##### block\_num

`number`

#### Returns

`NoteExecutionHint`

***

### always()

> `static` **always**(): `NoteExecutionHint`

Creates a hint indicating the note can always be consumed.

#### Returns

`NoteExecutionHint`

***

### fromParts()

> `static` **fromParts**(`tag`, `payload`): `NoteExecutionHint`

Reconstructs a hint from its encoded tag and payload.

#### Parameters

##### tag

`number`

##### payload

`number`

#### Returns

`NoteExecutionHint`

***

### none()

> `static` **none**(): `NoteExecutionHint`

Creates a hint that does not specify any execution constraint.

#### Returns

`NoteExecutionHint`

***

### onBlockSlot()

> `static` **onBlockSlot**(`epoch_len`, `slot_len`, `slot_offset`): `NoteExecutionHint`

Creates a hint that allows execution in a specific slot of a round.

#### Parameters

##### epoch\_len

`number`

##### slot\_len

`number`

##### slot\_offset

`number`

#### Returns

`NoteExecutionHint`
