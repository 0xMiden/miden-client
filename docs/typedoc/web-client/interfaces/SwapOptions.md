[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / SwapOptions

# Interface: SwapOptions

## Extends

- [`TransactionOptions`](TransactionOptions.md)

## Properties

### account

> **account**: [`AccountRef`](../type-aliases/AccountRef.md)

***

### offer

> **offer**: [`Asset`](Asset.md)

***

### paybackType?

> `optional` **paybackType**: [`NoteVisibility`](../type-aliases/NoteVisibility.md)

***

### prover?

> `optional` **prover**: [`TransactionProver`](../classes/TransactionProver.md)

Override default prover.

#### Inherited from

[`TransactionOptions`](TransactionOptions.md).[`prover`](TransactionOptions.md#prover)

***

### request

> **request**: [`Asset`](Asset.md)

***

### timeout?

> `optional` **timeout**: `number`

Wall-clock polling timeout in milliseconds for waitFor() (default: 60_000).
This is NOT a block height. For block-height-based parameters, see
`reclaimAfter` and `timelockUntil` on SendOptions.

#### Inherited from

[`TransactionOptions`](TransactionOptions.md).[`timeout`](TransactionOptions.md#timeout)

***

### type?

> `optional` **type**: [`NoteVisibility`](../type-aliases/NoteVisibility.md)

***

### waitForConfirmation?

> `optional` **waitForConfirmation**: `boolean`

#### Inherited from

[`TransactionOptions`](TransactionOptions.md).[`waitForConfirmation`](TransactionOptions.md#waitforconfirmation)
