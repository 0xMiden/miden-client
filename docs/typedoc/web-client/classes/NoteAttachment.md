[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / NoteAttachment

# Class: NoteAttachment

An attachment to a note.

Note attachments provide additional context about how notes should be processed.
For example, a network account target attachment indicates that the note should
be consumed by a specific network account.

## Constructors

### Constructor

> **new NoteAttachment**(): `NoteAttachment`

Creates a default (empty) note attachment.

#### Returns

`NoteAttachment`

## Methods

### \[dispose\]()

> **\[dispose\]**(): `void`

#### Returns

`void`

***

### asArray()

> **asArray**(): [`FeltArray`](FeltArray.md)

Returns the content as an array of Felts if the attachment kind is Array, otherwise None.

#### Returns

[`FeltArray`](FeltArray.md)

***

### asWord()

> **asWord**(): [`Word`](Word.md)

Returns the content as a Word if the attachment kind is Word, otherwise None.

#### Returns

[`Word`](Word.md)

***

### attachmentKind()

> **attachmentKind**(): [`NoteAttachmentKind`](../enumerations/NoteAttachmentKind.md)

Returns the attachment kind.

#### Returns

[`NoteAttachmentKind`](../enumerations/NoteAttachmentKind.md)

***

### attachmentScheme()

> **attachmentScheme**(): [`NoteAttachmentScheme`](NoteAttachmentScheme.md)

Returns the attachment scheme.

#### Returns

[`NoteAttachmentScheme`](NoteAttachmentScheme.md)

***

### free()

> **free**(): `void`

#### Returns

`void`

***

### newArray()

> `static` **newArray**(`scheme`, `elements`): `NoteAttachment`

Creates a new note attachment with Array content from the provided elements.

#### Parameters

##### scheme

[`NoteAttachmentScheme`](NoteAttachmentScheme.md)

##### elements

[`FeltArray`](FeltArray.md)

#### Returns

`NoteAttachment`

***

### newNetworkAccountTarget()

> `static` **newNetworkAccountTarget**(`target_id`, `exec_hint`): `NoteAttachment`

Creates a new note attachment for a network account target.

This attachment indicates that the note should be consumed by a specific network account.
Network accounts are accounts whose storage mode is `Network`, meaning the network (nodes)
can execute transactions on behalf of the account.

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

`NoteAttachment`

***

### newWord()

> `static` **newWord**(`scheme`, `word`): `NoteAttachment`

Creates a new note attachment with Word content from the provided word.

#### Parameters

##### scheme

[`NoteAttachmentScheme`](NoteAttachmentScheme.md)

##### word

[`Word`](Word.md)

#### Returns

`NoteAttachment`
