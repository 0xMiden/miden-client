[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / NoteTag

# Class: NoteTag

Note tags are best-effort filters for notes registered with the network. They hint whether a
note is meant for network or local execution and optionally embed a target (like part of an
`AccountId`) or a use-case payload. Public notes are required for network execution so that full
details are available for validation.

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

***

### fromHex()

> `static` **fromHex**(`hex`): `NoteTag`

Builds a note tag from a hex-encoded string (with or without 0x prefix).

#### Parameters

##### hex

`string`

#### Returns

`NoteTag`

***

### fromU32()

> `static` **fromU32**(`raw`): `NoteTag`

Builds a note tag from its raw u32 representation.

#### Parameters

##### raw

`number`

#### Returns

`NoteTag`
