[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / NetworkAccountTarget

# Class: NetworkAccountTarget

A standard note attachment that indicates a note should be consumed by a
specific network account.

Network accounts are accounts whose storage mode is `Network`, meaning the
network (nodes) can execute transactions on behalf of the account.

## Constructors

### Constructor

> **new NetworkAccountTarget**(`target_id`, `exec_hint`): `NetworkAccountTarget`

Creates a new network account target attachment.

# Arguments
* `target_id` - The ID of the network account that should consume the note
* `exec_hint` - A hint about when the note can be executed

# Errors
Returns an error if the target account is not a network account.

#### Parameters

##### target\_id

[`AccountId`](AccountId.md)

##### exec\_hint

[`NoteExecutionHint`](NoteExecutionHint.md)

#### Returns

`NetworkAccountTarget`

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

### intoAttachment()

> **intoAttachment**(): [`NoteAttachment`](NoteAttachment.md)

Converts this target into a note attachment.

#### Returns

[`NoteAttachment`](NoteAttachment.md)

***

### targetId()

> **targetId**(): [`AccountId`](AccountId.md)

Returns the ID of the target network account.

#### Returns

[`AccountId`](AccountId.md)
