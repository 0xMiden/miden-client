[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / ExecutedTransaction

# Class: ExecutedTransaction

Describes the result of executing a transaction program for the Miden protocol.

Executed transaction serves two primary purposes:
- It contains a complete description of the effects of the transaction. Specifically, it
  contains all output notes created as the result of the transaction and describes all the
  changes made to the involved account (i.e., the account delta).
- It contains all the information required to re-execute and prove the transaction in a
  stateless manner. This includes all public transaction inputs, but also all nondeterministic
  inputs that the host provided to Miden VM while executing the transaction (i.e., advice
  witness).

## Methods

### \[dispose\]()

> **\[dispose\]**(): `void`

#### Returns

`void`

***

### accountDelta()

> **accountDelta**(): [`AccountDelta`](AccountDelta.md)

Returns the account delta resulting from execution.

#### Returns

[`AccountDelta`](AccountDelta.md)

***

### accountId()

> **accountId**(): [`AccountId`](AccountId.md)

Returns the account the transaction was executed against.

#### Returns

[`AccountId`](AccountId.md)

***

### blockHeader()

> **blockHeader**(): [`BlockHeader`](BlockHeader.md)

Returns the block header that included the transaction.

#### Returns

[`BlockHeader`](BlockHeader.md)

***

### finalAccountHeader()

> **finalAccountHeader**(): [`AccountHeader`](AccountHeader.md)

Returns the final account header after execution.

#### Returns

[`AccountHeader`](AccountHeader.md)

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

### initialAccountHeader()

> **initialAccountHeader**(): [`AccountHeader`](AccountHeader.md)

Returns the initial account header before execution.

#### Returns

[`AccountHeader`](AccountHeader.md)

***

### inputNotes()

> **inputNotes**(): [`InputNotes`](InputNotes.md)

Returns the input notes consumed by the transaction.

#### Returns

[`InputNotes`](InputNotes.md)

***

### outputNotes()

> **outputNotes**(): [`OutputNotes`](OutputNotes.md)

Returns the output notes produced by the transaction.

#### Returns

[`OutputNotes`](OutputNotes.md)

***

### txArgs()

> **txArgs**(): [`TransactionArgs`](TransactionArgs.md)

Returns the arguments passed to the transaction script.

#### Returns

[`TransactionArgs`](TransactionArgs.md)
