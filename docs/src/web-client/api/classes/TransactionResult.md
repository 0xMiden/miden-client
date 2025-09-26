---
title: TransactionResult
draft: true
---

[**@demox-labs/miden-sdk**](../index)

***

[@demox-labs/miden-sdk](../index) / TransactionResult

# Class: TransactionResult

## Methods

### accountDelta()

> **accountDelta**(): [`AccountDelta`](AccountDelta)

#### Returns

[`AccountDelta`](AccountDelta)

***

### blockNum()

> **blockNum**(): `number`

#### Returns

`number`

***

### consumedNotes()

> **consumedNotes**(): [`InputNotes`](InputNotes)

#### Returns

[`InputNotes`](InputNotes)

***

### createdNotes()

> **createdNotes**(): [`OutputNotes`](OutputNotes)

#### Returns

[`OutputNotes`](OutputNotes)

***

### executedTransaction()

> **executedTransaction**(): [`ExecutedTransaction`](ExecutedTransaction)

#### Returns

[`ExecutedTransaction`](ExecutedTransaction)

***

### free()

> **free**(): `void`

#### Returns

`void`

***

### serialize()

> **serialize**(): `Uint8Array`

#### Returns

`Uint8Array`

***

### transactionArguments()

> **transactionArguments**(): [`TransactionArgs`](TransactionArgs)

#### Returns

[`TransactionArgs`](TransactionArgs)

***

### deserialize()

> `static` **deserialize**(`bytes`): `TransactionResult`

#### Parameters

##### bytes

`Uint8Array`

#### Returns

`TransactionResult`
