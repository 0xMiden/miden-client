[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / RpcClient

# Class: RpcClient

RPC Client for interacting with Miden nodes directly.

## Constructors

### Constructor

> **new RpcClient**(`endpoint`): `RpcClient`

Creates a new RPC client instance.

#### Parameters

##### endpoint

[`Endpoint`](Endpoint.md)

Endpoint to connect to.

#### Returns

`RpcClient`

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

### getNotesById()

> **getNotesById**(`note_ids`): `Promise`\<[`FetchedNote`](FetchedNote.md)[]\>

Fetches notes by their IDs from the connected Miden node.

#### Parameters

##### note\_ids

[`NoteId`](NoteId.md)[]

Array of [`NoteId`] objects to fetch

#### Returns

`Promise`\<[`FetchedNote`](FetchedNote.md)[]\>

Promise that resolves to different data depending on the note type:
- Private notes: Returns only `note_id` and `metadata` (from the note header). The
  `input_note` field will be `null`.
- Public notes: Returns the full `input_note` with inclusion proof, alongside metadata and
  ID.

***

### getNoteScriptByRoot()

> **getNoteScriptByRoot**(`script_root`): `Promise`\<[`NoteScript`](NoteScript.md)\>

Fetches a note script by its root hash from the connected Miden node.

#### Parameters

##### script\_root

[`Word`](Word.md)

#### Returns

`Promise`\<[`NoteScript`](NoteScript.md)\>

Promise that resolves to the `NoteScript`.
