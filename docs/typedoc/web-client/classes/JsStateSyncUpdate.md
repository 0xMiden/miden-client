[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / JsStateSyncUpdate

# Class: JsStateSyncUpdate

An object that contains data for a sync update,
which will be received by the applyStateSync JS function.
under sync.js

## Properties

### accountUpdates

> **accountUpdates**: [`JsAccountUpdate`](JsAccountUpdate.md)[]

Account state updates included in this sync.

***

### blockHasRelevantNotes

> **blockHasRelevantNotes**: `Uint8Array`

For each block in this update, stores a boolean (as u8) indicating whether
that block contains notes relevant to this client. Index i corresponds to
the ith block, with 1 meaning relevant and 0 meaning not relevant.

***

### blockNum

> **blockNum**: `string`

The block number for this update, stored as a string since it will be
persisted in `IndexedDB`.

***

### committedNoteIds

> **committedNoteIds**: `string`[]

IDs of note tags that should be removed from the client's local state.

***

### flattenedNewBlockHeaders

> **flattenedNewBlockHeaders**: [`FlattenedU8Vec`](FlattenedU8Vec.md)

The new block headers for this state update, serialized into a flattened byte array.

***

### flattenedPartialBlockChainPeaks

> **flattenedPartialBlockChainPeaks**: [`FlattenedU8Vec`](FlattenedU8Vec.md)

Flattened byte array containing partial blockchain peaks used for merkle tree
verification.

***

### newBlockNums

> **newBlockNums**: `string`[]

The block numbers corresponding to each header in `flattened_new_block_headers`.
This vec should have the same length as the number of headers, with each index
representing the block number for the header at that same index.

***

### serializedInputNotes

> **serializedInputNotes**: [`SerializedInputNoteData`](SerializedInputNoteData.md)[]

Input notes for this state update in serialized form.

***

### serializedNodeIds

> **serializedNodeIds**: `string`[]

Serialized IDs for new authentication nodes required to verify block headers.

***

### serializedNodes

> **serializedNodes**: `string`[]

The actual authentication node data corresponding to the IDs above.

***

### serializedOutputNotes

> **serializedOutputNotes**: [`SerializedOutputNoteData`](SerializedOutputNoteData.md)[]

Output notes created in this state update in serialized form.

***

### transactionUpdates

> **transactionUpdates**: [`SerializedTransactionData`](SerializedTransactionData.md)[]

Transaction data for transactions included in this update.

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
