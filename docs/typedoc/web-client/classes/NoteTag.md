[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / NoteTag

# Class: NoteTag

Compact tag used to categorize notes by issuer and use case.

## Methods

### asU32()

> **asU32**(): `number`

Returns the tag encoded as a `u32`.

#### Returns

`number`

***

### executionMode()

> **executionMode**(): [`NoteExecutionMode`](NoteExecutionMode.md)

Returns the execution mode encoded in the tag.

#### Returns

[`NoteExecutionMode`](NoteExecutionMode.md)

***

### free()

> **free**(): `void`

#### Returns

`void`

***

### isSingleTarget()

> **isSingleTarget**(): `boolean`

Returns `true` if the tag represents a single target note.

#### Returns

`boolean`

***

### forLocalUseCase()

> `static` **forLocalUseCase**(`use_case_id`, `payload`): `NoteTag`

Builds a tag for a local-only use case.

#### Parameters

##### use\_case\_id

`number`

##### payload

`number`

#### Returns

`NoteTag`

***

### forPublicUseCase()

> `static` **forPublicUseCase**(`use_case_id`, `payload`, `execution`): `NoteTag`

Builds a tag for a public use case with the given payload and execution mode.

#### Parameters

##### use\_case\_id

`number`

##### payload

`number`

##### execution

[`NoteExecutionMode`](NoteExecutionMode.md)

#### Returns

`NoteTag`

***

### fromAccountId()

> `static` **fromAccountId**(`account_id`): `NoteTag`

Builds a note tag tied to a specific account identifier.

#### Parameters

##### account\_id

[`AccountId`](AccountId.md)

#### Returns

`NoteTag`
