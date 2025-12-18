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
- Private notes: Returns the `noteHeader`, and the  `inclusionProof`. The `note` field will
  be `null`.
- Public notes: Returns the full `note` with `inclusionProof`, alongside its header.

***

### getNoteScriptByRoot()

> **getNoteScriptByRoot**(`script_root`): `Promise`\<[`NoteScript`](NoteScript.md)\>

Fetches a note script by its root hash from the connected Miden node.

#### Parameters

##### script\_root

[`Word`](Word.md)

The root hash of the note script to fetch.

#### Returns

`Promise`\<[`NoteScript`](NoteScript.md)\>

Promise that resolves to the `NoteScript`.
