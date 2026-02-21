[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / ConsumeOptions

# Interface: ConsumeOptions

## Extends

- [`TransactionOptions`](TransactionOptions.md)

## Properties

### account

> **account**: [`AccountRef`](../type-aliases/AccountRef.md)

***

### notes

> **notes**: [`NoteInput`](../type-aliases/NoteInput.md) \| [`NoteInput`](../type-aliases/NoteInput.md)[]

***

### prover?

> `optional` **prover**: [`TransactionProver`](../classes/TransactionProver.md)

Override default prover.

#### Inherited from

[`TransactionOptions`](TransactionOptions.md).[`prover`](TransactionOptions.md#prover)

***

### timeout?

> `optional` **timeout**: `number`

Wall-clock polling timeout in milliseconds for waitFor() (default: 60_000).
This is NOT a block height. For block-height-based parameters, see
`reclaimAfter` and `timelockUntil` on SendOptions.

#### Inherited from

[`TransactionOptions`](TransactionOptions.md).[`timeout`](TransactionOptions.md#timeout)

***

### waitForConfirmation?

> `optional` **waitForConfirmation**: `boolean`

#### Inherited from

[`TransactionOptions`](TransactionOptions.md).[`waitForConfirmation`](TransactionOptions.md#waitforconfirmation)
