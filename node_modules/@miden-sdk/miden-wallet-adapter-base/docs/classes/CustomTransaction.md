[**@miden-sdk/miden-wallet-adapter-base**](../README.md)

***

[@miden-sdk/miden-wallet-adapter-base](../README.md) / CustomTransaction

# Class: CustomTransaction

## Implements

- [`MidenCustomTransaction`](../interfaces/MidenCustomTransaction.md)

## Constructors

### Constructor

> **new CustomTransaction**(`address`, `recipientAddress`, `transactionRequest`, `inputNotesIds?`, `inputNoteBytes?`): `CustomTransaction`

#### Parameters

##### address

`string`

##### recipientAddress

`string`

##### transactionRequest

`TransactionRequest`

##### inputNotesIds?

`string`[]

##### inputNoteBytes?

`Uint8Array`\<`ArrayBufferLike`\>[]

#### Returns

`CustomTransaction`

## Properties

### address

> **address**: `string`

#### Implementation of

[`MidenCustomTransaction`](../interfaces/MidenCustomTransaction.md).[`address`](../interfaces/MidenCustomTransaction.md#address)

***

### importNotes?

> `optional` **importNotes**: `string`[]

#### Implementation of

[`MidenCustomTransaction`](../interfaces/MidenCustomTransaction.md).[`importNotes`](../interfaces/MidenCustomTransaction.md#importnotes)

***

### inputNoteIds?

> `optional` **inputNoteIds**: `string`[]

#### Implementation of

[`MidenCustomTransaction`](../interfaces/MidenCustomTransaction.md).[`inputNoteIds`](../interfaces/MidenCustomTransaction.md#inputnoteids)

***

### recipientAddress

> **recipientAddress**: `string`

#### Implementation of

[`MidenCustomTransaction`](../interfaces/MidenCustomTransaction.md).[`recipientAddress`](../interfaces/MidenCustomTransaction.md#recipientaddress)

***

### transactionRequest

> **transactionRequest**: `string`

#### Implementation of

[`MidenCustomTransaction`](../interfaces/MidenCustomTransaction.md).[`transactionRequest`](../interfaces/MidenCustomTransaction.md#transactionrequest)
