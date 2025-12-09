[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / SyncSummary

# Class: SyncSummary

## Methods

### \[dispose\]()

> **\[dispose\]**(): `void`

#### Returns

`void`

***

### blockNum()

> **blockNum**(): `number`

Returns the block height the summary is based on.

#### Returns

`number`

***

### committedNotes()

> **committedNotes**(): [`NoteId`](NoteId.md)[]

Returns IDs of notes committed in this sync window.

#### Returns

[`NoteId`](NoteId.md)[]

***

### committedTransactions()

> **committedTransactions**(): [`TransactionId`](TransactionId.md)[]

Returns transactions that were committed.

#### Returns

[`TransactionId`](TransactionId.md)[]

***

### consumedNotes()

> **consumedNotes**(): [`NoteId`](NoteId.md)[]

Returns IDs of notes that were consumed.

#### Returns

[`NoteId`](NoteId.md)[]

***

### free()

> **free**(): `void`

#### Returns

`void`

***

### serialize()

> **serialize**(): `Uint8Array`

Serializes the sync summary into bytes.

#### Returns

`Uint8Array`

***

### updatedAccounts()

> **updatedAccounts**(): [`AccountId`](AccountId.md)[]

Returns accounts that were updated.

#### Returns

[`AccountId`](AccountId.md)[]

***

### deserialize()

> `static` **deserialize**(`bytes`): `SyncSummary`

Deserializes a sync summary from bytes.

#### Parameters

##### bytes

`Uint8Array`

#### Returns

`SyncSummary`
