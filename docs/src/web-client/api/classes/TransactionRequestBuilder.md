---
title: TransactionRequestBuilder
draft: true
---

[**@demox-labs/miden-sdk**](../index)

***

[@demox-labs/miden-sdk](../index) / TransactionRequestBuilder

# Class: TransactionRequestBuilder

## Constructors

### Constructor

> **new TransactionRequestBuilder**(): `TransactionRequestBuilder`

#### Returns

`TransactionRequestBuilder`

## Methods

### build()

> **build**(): [`TransactionRequest`](TransactionRequest)

#### Returns

[`TransactionRequest`](TransactionRequest)

***

### extendAdviceMap()

> **extendAdviceMap**(`advice_map`): `TransactionRequestBuilder`

#### Parameters

##### advice\_map

[`AdviceMap`](AdviceMap)

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

[`Word`](Word)

#### Returns

`TransactionRequestBuilder`

***

### withAuthenticatedInputNotes()

> **withAuthenticatedInputNotes**(`notes`): `TransactionRequestBuilder`

#### Parameters

##### notes

[`NoteIdAndArgsArray`](NoteIdAndArgsArray)

#### Returns

`TransactionRequestBuilder`

***

### withCustomScript()

> **withCustomScript**(`script`): `TransactionRequestBuilder`

#### Parameters

##### script

[`TransactionScript`](TransactionScript)

#### Returns

`TransactionRequestBuilder`

***

### withExpectedFutureNotes()

> **withExpectedFutureNotes**(`note_details_and_tag`): `TransactionRequestBuilder`

#### Parameters

##### note\_details\_and\_tag

[`NoteDetailsAndTagArray`](NoteDetailsAndTagArray)

#### Returns

`TransactionRequestBuilder`

***

### withExpectedOutputRecipients()

> **withExpectedOutputRecipients**(`recipients`): `TransactionRequestBuilder`

#### Parameters

##### recipients

[`RecipientArray`](RecipientArray)

#### Returns

`TransactionRequestBuilder`

***

### withForeignAccounts()

> **withForeignAccounts**(`foreign_accounts`): `TransactionRequestBuilder`

#### Parameters

##### foreign\_accounts

[`ForeignAccount`](ForeignAccount)[]

#### Returns

`TransactionRequestBuilder`

***

### withOwnOutputNotes()

> **withOwnOutputNotes**(`notes`): `TransactionRequestBuilder`

#### Parameters

##### notes

[`OutputNotesArray`](OutputNotesArray)

#### Returns

`TransactionRequestBuilder`

***

### withScriptArg()

> **withScriptArg**(`script_arg`): `TransactionRequestBuilder`

#### Parameters

##### script\_arg

[`Word`](Word)

#### Returns

`TransactionRequestBuilder`

***

### withUnauthenticatedInputNotes()

> **withUnauthenticatedInputNotes**(`notes`): `TransactionRequestBuilder`

#### Parameters

##### notes

[`NoteAndArgsArray`](NoteAndArgsArray)

#### Returns

`TransactionRequestBuilder`
