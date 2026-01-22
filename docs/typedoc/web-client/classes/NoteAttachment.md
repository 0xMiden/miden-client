[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / NoteAttachment

# Class: NoteAttachment

An attachment to a note.

Note attachments provide additional context about how notes should be processed.
For example, a network account target attachment indicates that the note should
be consumed by a specific network account.

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

### newWord()

> `static` **newWord**(`word`): `NoteAttachment`

Creates a new note attachment with no scheme (scheme = 0).

#### Parameters

##### word

[`Word`](Word.md)

#### Returns

`NoteAttachment`
