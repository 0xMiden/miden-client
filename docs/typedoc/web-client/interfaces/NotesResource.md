[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / NotesResource

# Interface: NotesResource

## Methods

### export()

> **export**(`noteId`, `options?`): `Promise`\<[`NoteFile`](../classes/NoteFile.md)\>

#### Parameters

##### noteId

`string`

##### options?

[`ExportNoteOptions`](ExportNoteOptions.md)

#### Returns

`Promise`\<[`NoteFile`](../classes/NoteFile.md)\>

***

### fetch()

> **fetch**(`options?`): `Promise`\<`void`\>

#### Parameters

##### options?

[`FetchPrivateNotesOptions`](FetchPrivateNotesOptions.md)

#### Returns

`Promise`\<`void`\>

***

### get()

> **get**(`noteId`): `Promise`\<[`InputNoteRecord`](../classes/InputNoteRecord.md)\>

#### Parameters

##### noteId

`string`

#### Returns

`Promise`\<[`InputNoteRecord`](../classes/InputNoteRecord.md)\>

***

### import()

> **import**(`noteFile`): `Promise`\<[`NoteId`](../classes/NoteId.md)\>

#### Parameters

##### noteFile

[`NoteFile`](../classes/NoteFile.md)

#### Returns

`Promise`\<[`NoteId`](../classes/NoteId.md)\>

***

### list()

> **list**(`query?`): `Promise`\<[`InputNoteRecord`](../classes/InputNoteRecord.md)[]\>

#### Parameters

##### query?

[`NoteQuery`](../type-aliases/NoteQuery.md)

#### Returns

`Promise`\<[`InputNoteRecord`](../classes/InputNoteRecord.md)[]\>

***

### listAvailable()

> **listAvailable**(`options`): `Promise`\<[`ConsumableNoteRecord`](../classes/ConsumableNoteRecord.md)[]\>

#### Parameters

##### options

###### account

[`AccountRef`](../type-aliases/AccountRef.md)

#### Returns

`Promise`\<[`ConsumableNoteRecord`](../classes/ConsumableNoteRecord.md)[]\>

***

### listSent()

> **listSent**(`query?`): `Promise`\<[`OutputNoteRecord`](../classes/OutputNoteRecord.md)[]\>

#### Parameters

##### query?

[`NoteQuery`](../type-aliases/NoteQuery.md)

#### Returns

`Promise`\<[`OutputNoteRecord`](../classes/OutputNoteRecord.md)[]\>

***

### sendPrivate()

> **sendPrivate**(`options`): `Promise`\<`void`\>

#### Parameters

##### options

[`SendPrivateOptions`](SendPrivateOptions.md)

#### Returns

`Promise`\<`void`\>
