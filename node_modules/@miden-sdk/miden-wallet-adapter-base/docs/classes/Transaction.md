[**@miden-sdk/miden-wallet-adapter-base**](../README.md)

***

[@miden-sdk/miden-wallet-adapter-base](../README.md) / Transaction

# Class: Transaction

## Implements

- [`MidenTransaction`](../interfaces/MidenTransaction.md)

## Constructors

### Constructor

> **new Transaction**(`type`, `payload`): `Transaction`

#### Parameters

##### type

[`TransactionType`](../enumerations/TransactionType.md)

##### payload

[`TransactionPayload`](../type-aliases/TransactionPayload.md)

#### Returns

`Transaction`

## Properties

### payload

> **payload**: [`TransactionPayload`](../type-aliases/TransactionPayload.md)

#### Implementation of

[`MidenTransaction`](../interfaces/MidenTransaction.md).[`payload`](../interfaces/MidenTransaction.md#payload)

***

### type

> **type**: [`TransactionType`](../enumerations/TransactionType.md)

#### Implementation of

[`MidenTransaction`](../interfaces/MidenTransaction.md).[`type`](../interfaces/MidenTransaction.md#type)

## Methods

### createConsumeTransaction()

> `static` **createConsumeTransaction**(`faucetId`, `noteId`, `noteType`, `amount`, `noteBytes?`): `Transaction`

#### Parameters

##### faucetId

`string`

##### noteId

`string`

##### noteType

[`NoteTypeString`](../type-aliases/NoteTypeString.md)

##### amount

`number`

##### noteBytes?

`Uint8Array`

#### Returns

`Transaction`

***

### createCustomTransaction()

> `static` **createCustomTransaction**(`address`, `recipientAddress`, `transactionRequest`, `inputNoteIds?`, `noteBytes?`): `Transaction`

#### Parameters

##### address

`string`

##### recipientAddress

`string`

##### transactionRequest

`TransactionRequest`

##### inputNoteIds?

`string`[]

##### noteBytes?

`Uint8Array`\<`ArrayBufferLike`\>[]

#### Returns

`Transaction`

***

### createSendTransaction()

> `static` **createSendTransaction**(`sender`, `recipient`, `faucetId`, `noteType`, `amount`, `recallBlocks?`): `Transaction`

#### Parameters

##### sender

`string`

##### recipient

`string`

##### faucetId

`string`

##### noteType

[`NoteTypeString`](../type-aliases/NoteTypeString.md)

##### amount

`number`

##### recallBlocks?

`number`

#### Returns

`Transaction`
