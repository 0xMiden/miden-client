[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / TransactionRequestBuilder

# Class: TransactionRequestBuilder

Fluent builder for assembling [`TransactionRequest`] instances from JavaScript.

## Constructors

### Constructor

> **new TransactionRequestBuilder**(): `TransactionRequestBuilder`

Creates an empty transaction request builder.

#### Returns

`TransactionRequestBuilder`

## Methods

### build()

> **build**(): [`TransactionRequest`](TransactionRequest.md)

Builds the transaction request.

#### Returns

[`TransactionRequest`](TransactionRequest.md)

***

### extendAdviceMap()

> **extendAdviceMap**(`advice_map`): `TransactionRequestBuilder`

Adds additional advice inputs to the transaction.

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

Sets the authentication argument for the transaction script.

#### Parameters

##### auth\_arg

[`Word`](Word.md)

#### Returns

`TransactionRequestBuilder`

***

### withAuthenticatedInputNotes()

> **withAuthenticatedInputNotes**(`notes`): `TransactionRequestBuilder`

Adds authenticated input notes to the request.

#### Parameters

##### notes

[`NoteIdAndArgsArray`](NoteIdAndArgsArray.md)

#### Returns

`TransactionRequestBuilder`

***

### withCustomScript()

> **withCustomScript**(`script`): `TransactionRequestBuilder`

Overrides the default transaction script.

#### Parameters

##### script

[`TransactionScript`](TransactionScript.md)

#### Returns

`TransactionRequestBuilder`

***

### withExpectedFutureNotes()

> **withExpectedFutureNotes**(`note_details_and_tag`): `TransactionRequestBuilder`

Declares future notes the transaction expects to create.

#### Parameters

##### note\_details\_and\_tag

[`NoteDetailsAndTagArray`](NoteDetailsAndTagArray.md)

#### Returns

`TransactionRequestBuilder`

***

### withExpectedOutputRecipients()

> **withExpectedOutputRecipients**(`recipients`): `TransactionRequestBuilder`

Declares the recipients expected to receive notes from the transaction.

#### Parameters

##### recipients

[`RecipientArray`](RecipientArray.md)

#### Returns

`TransactionRequestBuilder`

***

### withForeignAccounts()

> **withForeignAccounts**(`foreign_accounts`): `TransactionRequestBuilder`

Attaches foreign accounts required for script execution.

#### Parameters

##### foreign\_accounts

[`ForeignAccount`](ForeignAccount.md)[]

#### Returns

`TransactionRequestBuilder`

***

### withOwnOutputNotes()

> **withOwnOutputNotes**(`notes`): `TransactionRequestBuilder`

Specifies the output notes owned by the originating account.

#### Parameters

##### notes

[`OutputNotesArray`](OutputNotesArray.md)

#### Returns

`TransactionRequestBuilder`

***

### withScriptArg()

> **withScriptArg**(`script_arg`): `TransactionRequestBuilder`

Sets the script argument for the transaction script.

#### Parameters

##### script\_arg

[`Word`](Word.md)

#### Returns

`TransactionRequestBuilder`

***

### withUnauthenticatedInputNotes()

> **withUnauthenticatedInputNotes**(`notes`): `TransactionRequestBuilder`

Adds unauthenticated input notes to the request.

#### Parameters

##### notes

[`NoteAndArgsArray`](NoteAndArgsArray.md)

#### Returns

`TransactionRequestBuilder`
