[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / NoteConsumptionStatus

# Class: NoteConsumptionStatus

Describes if a note could be consumed under a specific conditions: target account state and
block height.

## Methods

### \[dispose\]()

> **\[dispose\]**(): `void`

#### Returns

`void`

***

### consumableAfterBlock()

> **consumableAfterBlock**(): `number`

Returns the block number at which the note can be consumed.
Returns None if the note is already consumable or never possible

#### Returns

`number`

***

### free()

> **free**(): `void`

#### Returns

`void`

***

### consumable()

> `static` **consumable**(): `NoteConsumptionStatus`

Constructs a `NoteConsumptionStatus` that is consumable.

#### Returns

`NoteConsumptionStatus`

***

### consumableAfter()

> `static` **consumableAfter**(`block_height`): `NoteConsumptionStatus`

Constructs a `NoteConsumptionStatus` that is consumable after a specific block height.

#### Parameters

##### block\_height

`number`

#### Returns

`NoteConsumptionStatus`

***

### consumableWithAuthorization()

> `static` **consumableWithAuthorization**(): `NoteConsumptionStatus`

Constructs a `NoteConsumptionStatus` that is consumable with authorization.

#### Returns

`NoteConsumptionStatus`

***

### neverConsumable()

> `static` **neverConsumable**(`err`): `NoteConsumptionStatus`

Constructs a `NoteConsumptionStatus` that is never consumable.

#### Parameters

##### err

`string`

#### Returns

`NoteConsumptionStatus`

***

### unconsumableConditions()

> `static` **unconsumableConditions**(): `NoteConsumptionStatus`

Constructs a `NoteConsumptionStatus` that is unconsumable due to conditions.

#### Returns

`NoteConsumptionStatus`
