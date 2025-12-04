[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / TransactionRequestBuilder

# Class: TransactionRequestBuilder

A builder for a [`TransactionRequest`].

Use this builder to construct a [`TransactionRequest`] by adding input notes, specifying
scripts, and setting other transaction parameters.

## Constructors

### Constructor

> **new TransactionRequestBuilder**(): `TransactionRequestBuilder`

Creates a new empty transaction request builder.

#### Returns

`TransactionRequestBuilder`

## Methods

### \[dispose\]()

> **\[dispose\]**(): `void`

#### Returns

`void`

***

### build()

> **build**(): [`TransactionRequest`](TransactionRequest.md)

Finalizes the builder into a `TransactionRequest`.

#### Returns

[`TransactionRequest`](TransactionRequest.md)

***

### extendAdviceMap()

> **extendAdviceMap**(`advice_map`): `TransactionRequestBuilder`

Merges an advice map to be available during script execution.

#### Parameters

##### advice\_map

[`AdviceMap`](AdviceMap.md)

#### Returns

`TransactionRequestBuilder`

***

### free()

> **free**(): `void`

#### Returns

`void`

***

### withAuthArg()

> **withAuthArg**(`auth_arg`): `TransactionRequestBuilder`

Adds an authentication argument.

#### Parameters

##### auth\_arg

[`Word`](Word.md)

#### Returns

`TransactionRequestBuilder`

***

### withAuthenticatedInputNotes()

> **withAuthenticatedInputNotes**(`notes`): `TransactionRequestBuilder`

Adds authenticated input notes (identified by ID) with optional arguments.

#### Parameters

##### notes

[`NoteIdAndArgsArray`](NoteIdAndArgsArray.md)

#### Returns

`TransactionRequestBuilder`

***

### withCustomScript()

> **withCustomScript**(`script`): `TransactionRequestBuilder`

Attaches a custom transaction script.

#### Parameters

##### script

[`TransactionScript`](TransactionScript.md)

#### Returns

`TransactionRequestBuilder`

***

### withExpectedFutureNotes()

> **withExpectedFutureNotes**(`note_details_and_tag`): `TransactionRequestBuilder`

Declares notes expected to be created in follow-up executions.

#### Parameters

##### note\_details\_and\_tag

[`NoteDetailsAndTagArray`](NoteDetailsAndTagArray.md)

#### Returns

`TransactionRequestBuilder`

***

### withExpectedOutputRecipients()

> **withExpectedOutputRecipients**(`recipients`): `TransactionRequestBuilder`

Declares expected output recipients (used for verification).

#### Parameters

##### recipients

[`NoteRecipientArray`](NoteRecipientArray.md)

#### Returns

`TransactionRequestBuilder`

***

### withForeignAccounts()

> **withForeignAccounts**(`foreign_accounts`): `TransactionRequestBuilder`

Registers foreign accounts referenced by the transaction.

#### Parameters

##### foreign\_accounts

[`ForeignAccountArray`](ForeignAccountArray.md)

#### Returns

`TransactionRequestBuilder`

***

### withOwnOutputNotes()

> **withOwnOutputNotes**(`notes`): `TransactionRequestBuilder`

Adds notes created by the sender that should be emitted by the transaction.

#### Parameters

##### notes

[`OutputNoteArray`](OutputNoteArray.md)

#### Returns

`TransactionRequestBuilder`

***

### withScriptArg()

> **withScriptArg**(`script_arg`): `TransactionRequestBuilder`

Adds a transaction script argument.

#### Parameters

##### script\_arg

[`Word`](Word.md)

#### Returns

`TransactionRequestBuilder`

***

### withUnauthenticatedInputNotes()

> **withUnauthenticatedInputNotes**(`notes`): `TransactionRequestBuilder`

Adds unauthenticated input notes with optional arguments.

#### Parameters

##### notes

[`NoteAndArgsArray`](NoteAndArgsArray.md)

#### Returns

`TransactionRequestBuilder`
