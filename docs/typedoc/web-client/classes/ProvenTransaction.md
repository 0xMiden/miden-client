[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / ProvenTransaction

# Class: ProvenTransaction

## Methods

### \[dispose\]()

> **\[dispose\]**(): `void`

#### Returns

`void`

***

### accountId()

> **accountId**(): [`AccountId`](AccountId.md)

Returns the account ID the transaction was executed against.

#### Returns

[`AccountId`](AccountId.md)

***

### expirationBlockNumber()

> **expirationBlockNumber**(): `number`

Returns the block number at which the transaction expires.

#### Returns

`number`

***

### free()

> **free**(): `void`

#### Returns

`void`

***

### id()

> **id**(): [`TransactionId`](TransactionId.md)

Returns the transaction ID.

#### Returns

[`TransactionId`](TransactionId.md)

***

### nullifiers()

> **nullifiers**(): [`Word`](Word.md)[]

Returns the nullifiers of the consumed input notes.

#### Returns

[`Word`](Word.md)[]

***

### outputNotes()

> **outputNotes**(): [`OutputNotes`](OutputNotes.md)

Returns notes created by this transaction.

#### Returns

[`OutputNotes`](OutputNotes.md)

***

### refBlockCommitment()

> **refBlockCommitment**(): [`Word`](Word.md)

Returns the commitment of the reference block.

#### Returns

[`Word`](Word.md)

***

### refBlockNumber()

> **refBlockNumber**(): `number`

Returns the reference block number used during execution.

#### Returns

`number`

***

### serialize()

> **serialize**(): `Uint8Array`

Serializes the proven transaction into bytes.

#### Returns

`Uint8Array`

***

### deserialize()

> `static` **deserialize**(`bytes`): `ProvenTransaction`

Deserializes a proven transaction from bytes.

#### Parameters

##### bytes

`Uint8Array`

#### Returns

`ProvenTransaction`
