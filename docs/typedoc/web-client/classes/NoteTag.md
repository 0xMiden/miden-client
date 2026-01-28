[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / NoteTag

# Class: NoteTag

Note tags are 32-bits of data that serve as best-effort filters for notes.

Tags enable quick lookups for notes related to particular use cases, scripts, or account
prefixes.

## Constructors

### Constructor

> **new NoteTag**(`tag`): `NoteTag`

Creates a new `NoteTag` from an arbitrary u32.

#### Parameters

##### tag

`number`

#### Returns

`NoteTag`

## Methods

### \[dispose\]()

> **\[dispose\]**(): `void`

#### Returns

`void`

***

### asU32()

> **asU32**(): `number`

Returns the inner u32 value of this tag.

#### Returns

`number`

***

### free()

> **free**(): `void`

#### Returns

`void`

***

### withAccountTarget()

> `static` **withAccountTarget**(`account_id`): `NoteTag`

Constructs a note tag that targets the given account ID.

#### Parameters

##### account\_id

[`AccountId`](AccountId.md)

#### Returns

`NoteTag`

***

### withCustomAccountTarget()

> `static` **withCustomAccountTarget**(`account_id`, `tag_len`): `NoteTag`

Constructs a note tag that targets the given account ID with a custom tag length.

#### Parameters

##### account\_id

[`AccountId`](AccountId.md)

##### tag\_len

`number`

#### Returns

`NoteTag`
