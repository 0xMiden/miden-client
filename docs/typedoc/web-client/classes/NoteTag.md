[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / NoteTag

# Class: NoteTag

## Methods

### \[dispose\]()

> **\[dispose\]**(): `void`

#### Returns

`void`

***

### asU32()

> **asU32**(): `number`

Returns the underlying 32-bit representation.

#### Returns

`number`

***

### executionMode()

> **executionMode**(): [`NoteExecutionMode`](NoteExecutionMode.md)

Returns the execution mode encoded in this tag.

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

Returns true if the tag targets a single account.

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

Builds a tag for a public use case with an explicit payload and execution mode.

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

Builds a single-target tag derived from an account ID.

#### Parameters

##### account\_id

[`AccountId`](AccountId.md)

#### Returns

`NoteTag`
