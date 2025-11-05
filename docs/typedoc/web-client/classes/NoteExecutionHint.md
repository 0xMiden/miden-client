[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / NoteExecutionHint

# Class: NoteExecutionHint

Scheduling hint describing when a note becomes consumable.

## Methods

### canBeConsumed()

> **canBeConsumed**(`block_num`): `boolean`

Returns whether the hint allows consumption at the provided block.

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

Returns a hint that defers consumption until after the given block number.

#### Parameters

##### block\_num

`number`

#### Returns

`NoteExecutionHint`

***

### always()

> `static` **always**(): `NoteExecutionHint`

Returns a hint that allows consumption at any time.

#### Returns

`NoteExecutionHint`

***

### fromParts()

> `static` **fromParts**(`tag`, `payload`): `NoteExecutionHint`

Recreates a hint from low-level tag/payload components.

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

Returns a hint with no additional restrictions.

#### Returns

`NoteExecutionHint`

***

### onBlockSlot()

> `static` **onBlockSlot**(`epoch_len`, `slot_len`, `slot_offset`): `NoteExecutionHint`

Returns a hint that limits consumption to a specific slot schedule.

#### Parameters

##### epoch\_len

`number`

##### slot\_len

`number`

##### slot\_offset

`number`

#### Returns

`NoteExecutionHint`
