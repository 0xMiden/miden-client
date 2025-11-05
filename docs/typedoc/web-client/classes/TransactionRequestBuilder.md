[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / TransactionRequestBuilder

# Class: TransactionRequestBuilder

## Constructors

### Constructor

> **new TransactionRequestBuilder**(): `TransactionRequestBuilder`

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

#### Returns

[`TransactionRequest`](TransactionRequest.md)

***

### extendAdviceMap()

> **extendAdviceMap**(`advice_map`): `TransactionRequestBuilder`

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

#### Parameters

##### auth\_arg

[`Word`](Word.md)

#### Returns

`TransactionRequestBuilder`

***

### withAuthenticatedInputNotes()

> **withAuthenticatedInputNotes**(`notes`): `TransactionRequestBuilder`

#### Parameters

##### notes

[`NoteIdAndArgsArray`](NoteIdAndArgsArray.md)

#### Returns

`TransactionRequestBuilder`

***

### withCustomScript()

> **withCustomScript**(`script`): `TransactionRequestBuilder`

#### Parameters

##### script

[`TransactionScript`](TransactionScript.md)

#### Returns

`TransactionRequestBuilder`

***

### withExpectedFutureNotes()

> **withExpectedFutureNotes**(`note_details_and_tag`): `TransactionRequestBuilder`

#### Parameters

##### note\_details\_and\_tag

[`NoteDetailsAndTagArray`](NoteDetailsAndTagArray.md)

#### Returns

`TransactionRequestBuilder`

***

### withExpectedOutputRecipients()

> **withExpectedOutputRecipients**(`recipients`): `TransactionRequestBuilder`

#### Parameters

##### recipients

[`RecipientArray`](RecipientArray.md)

#### Returns

`TransactionRequestBuilder`

***

### withForeignAccounts()

> **withForeignAccounts**(`foreign_accounts`): `TransactionRequestBuilder`

#### Parameters

##### foreign\_accounts

[`ForeignAccount`](ForeignAccount.md)[]

#### Returns

`TransactionRequestBuilder`

***

### withOwnOutputNotes()

> **withOwnOutputNotes**(`notes`): `TransactionRequestBuilder`

#### Parameters

##### notes

[`OutputNotesArray`](OutputNotesArray.md)

#### Returns

`TransactionRequestBuilder`

***

### withScriptArg()

> **withScriptArg**(`script_arg`): `TransactionRequestBuilder`

#### Parameters

##### script\_arg

[`Word`](Word.md)

#### Returns

`TransactionRequestBuilder`

***

### withUnauthenticatedInputNotes()

> **withUnauthenticatedInputNotes**(`notes`): `TransactionRequestBuilder`

#### Parameters

##### notes

[`NoteAndArgsArray`](NoteAndArgsArray.md)

#### Returns

`TransactionRequestBuilder`
