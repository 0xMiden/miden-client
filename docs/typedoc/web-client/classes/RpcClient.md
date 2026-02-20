[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / RpcClient

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

### getAccountDetails()

> **getAccountDetails**(`account_id`): `Promise`\<[`FetchedAccount`](FetchedAccount.md)\>

Fetches account details for a specific account ID.

#### Parameters

##### account\_id

[`AccountId`](AccountId.md)

#### Returns

`Promise`\<[`FetchedAccount`](FetchedAccount.md)\>

***

### getAccountProof()

> **getAccountProof**(`account_id`): `Promise`\<[`AccountProof`](AccountProof.md)\>

Fetches account headers from the node for a public account.

This is a lighter-weight alternative to `getAccountDetails` that makes a single RPC call
and returns the account header, storage slot values, and account code without
reconstructing the full account state.

Useful for reading storage slot values (e.g., faucet metadata) without the overhead of
fetching the complete account with all vault assets and storage map entries.

#### Parameters

##### account\_id

[`AccountId`](AccountId.md)

#### Returns

`Promise`\<[`AccountProof`](AccountProof.md)\>

***

### getBlockHeaderByNumber()

> **getBlockHeaderByNumber**(`block_num?`): `Promise`\<[`BlockHeader`](BlockHeader.md)\>

Fetches a block header by number. When `block_num` is undefined, returns the latest header.

#### Parameters

##### block\_num?

`number`

#### Returns

`Promise`\<[`BlockHeader`](BlockHeader.md)\>

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

***

### getNullifierCommitHeight()

> **getNullifierCommitHeight**(`nullifier`, `block_num`): `Promise`\<`number`\>

Fetches the block height at which a nullifier was committed, if any.

#### Parameters

##### nullifier

[`Word`](Word.md)

##### block\_num

`number`

#### Returns

`Promise`\<`number`\>

***

### syncNotes()

> **syncNotes**(`block_num`, `block_to`, `note_tags`): `Promise`\<[`NoteSyncInfo`](NoteSyncInfo.md)\>

Fetches notes matching the provided tags from the node.

#### Parameters

##### block\_num

`number`

##### block\_to

`number`

##### note\_tags

[`NoteTag`](NoteTag.md)[]

#### Returns

`Promise`\<[`NoteSyncInfo`](NoteSyncInfo.md)\>
