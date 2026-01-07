[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / TransactionRecord

# Class: TransactionRecord

Describes a transaction that has been executed and is being tracked on the Client.

## Methods

### \[dispose\]()

> **\[dispose\]**(): `void`

#### Returns

`void`

***

### accountId()

> **accountId**(): [`AccountId`](AccountId.md)

Returns the account this transaction was executed against.

#### Returns

[`AccountId`](AccountId.md)

***

### blockNum()

> **blockNum**(): `number`

Returns the block height in which the transaction was included.

#### Returns

`number`

***

### creationTimestamp()

> **creationTimestamp**(): `bigint`

Returns the timestamp when the record was created.

#### Returns

`bigint`

***

### expirationBlockNum()

> **expirationBlockNum**(): `number`

Returns the expiration block height for the transaction.

#### Returns

`number`

***

### finalAccountState()

> **finalAccountState**(): [`Word`](Word.md)

Returns the final account state commitment after execution.

#### Returns

[`Word`](Word.md)

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

### initAccountState()

> **initAccountState**(): [`Word`](Word.md)

Returns the initial account state commitment before execution.

#### Returns

[`Word`](Word.md)

***

### inputNoteNullifiers()

> **inputNoteNullifiers**(): [`Word`](Word.md)[]

Returns the nullifiers of the consumed input notes.

#### Returns

[`Word`](Word.md)[]

***

### outputNotes()

> **outputNotes**(): [`OutputNotes`](OutputNotes.md)

Returns the output notes created by this transaction.

#### Returns

[`OutputNotes`](OutputNotes.md)

***

### submissionHeight()

> **submissionHeight**(): `number`

Returns the block height at which the transaction was submitted.

#### Returns

`number`

***

### transactionStatus()

> **transactionStatus**(): [`TransactionStatus`](TransactionStatus.md)

Returns the current status of the transaction.

#### Returns

[`TransactionStatus`](TransactionStatus.md)
