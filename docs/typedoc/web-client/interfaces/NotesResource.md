[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / NotesResource

# Interface: NotesResource

## Methods

### export()

> **export**(`noteId`, `options?`): `Promise`\<[`NoteFile`](../classes/NoteFile.md)\>

Export a note to a [NoteFile](../classes/NoteFile.md) for transfer or backup.

#### Parameters

##### noteId

[`NoteInput`](../type-aliases/NoteInput.md)

##### options?

[`ExportNoteOptions`](ExportNoteOptions.md)

#### Returns

`Promise`\<[`NoteFile`](../classes/NoteFile.md)\>

***

### fetchPrivate()

> **fetchPrivate**(`options?`): `Promise`\<`void`\>

Fetch private notes from the note transport service.

#### Parameters

##### options?

[`FetchPrivateNotesOptions`](FetchPrivateNotesOptions.md)

#### Returns

`Promise`\<`void`\>

***

### get()

> **get**(`noteId`): `Promise`\<[`InputNoteRecord`](../classes/InputNoteRecord.md)\>

Retrieve a note by ID. Returns `null` if not found.

#### Parameters

##### noteId

[`NoteInput`](../type-aliases/NoteInput.md)

#### Returns

`Promise`\<[`InputNoteRecord`](../classes/InputNoteRecord.md)\>

***

### import()

> **import**(`noteFile`): `Promise`\<[`NoteId`](../classes/NoteId.md)\>

Import a note from a [NoteFile](../classes/NoteFile.md).

#### Parameters

##### noteFile

[`NoteFile`](../classes/NoteFile.md)

#### Returns

`Promise`\<[`NoteId`](../classes/NoteId.md)\>

***

### list()

> **list**(`query?`): `Promise`\<[`InputNoteRecord`](../classes/InputNoteRecord.md)[]\>

List received (input) notes, optionally filtered by status or IDs.

#### Parameters

##### query?

[`NoteQuery`](../type-aliases/NoteQuery.md)

#### Returns

`Promise`\<[`InputNoteRecord`](../classes/InputNoteRecord.md)[]\>

***

### listAvailable()

> **listAvailable**(`options`): `Promise`\<[`InputNoteRecord`](../classes/InputNoteRecord.md)[]\>

List notes that are available for consumption by a specific account.

#### Parameters

##### options

###### account

[`AccountRef`](../type-aliases/AccountRef.md)

#### Returns

`Promise`\<[`InputNoteRecord`](../classes/InputNoteRecord.md)[]\>

***

### listSent()

> **listSent**(`query?`): `Promise`\<[`OutputNoteRecord`](../classes/OutputNoteRecord.md)[]\>

List sent (output) notes, optionally filtered by status or IDs.

#### Parameters

##### query?

[`NoteQuery`](../type-aliases/NoteQuery.md)

#### Returns

`Promise`\<[`OutputNoteRecord`](../classes/OutputNoteRecord.md)[]\>

***

### sendPrivate()

> **sendPrivate**(`options`): `Promise`\<`void`\>

Send a private note to a recipient via the note transport service.

#### Parameters

##### options

[`SendPrivateOptions`](SendPrivateOptions.md)

#### Returns

`Promise`\<`void`\>
