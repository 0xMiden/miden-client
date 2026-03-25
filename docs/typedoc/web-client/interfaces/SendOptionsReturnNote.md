[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / SendOptionsReturnNote

# Interface: SendOptionsReturnNote

## Extends

- [`TransactionOptions`](TransactionOptions.md)

## Properties

### account

> **account**: [`AccountRef`](../type-aliases/AccountRef.md)

***

### amount

> **amount**: `number` \| `bigint`

***

### prover?

> `optional` **prover**: [`TransactionProver`](../classes/TransactionProver.md)

Override default prover.

#### Inherited from

[`TransactionOptions`](TransactionOptions.md).[`prover`](TransactionOptions.md#prover)

***

### returnNote

> **returnNote**: `true`

***

### timeout?

> `optional` **timeout**: `number`

Wall-clock polling timeout in milliseconds for waitFor() (default: 60_000).
This is NOT a block height. For block-height-based parameters, see
`reclaimAfter` and `timelockUntil` on SendOptions.

#### Inherited from

[`TransactionOptions`](TransactionOptions.md).[`timeout`](TransactionOptions.md#timeout)

***

### to

> **to**: [`AccountRef`](../type-aliases/AccountRef.md)

***

### token

> **token**: [`AccountRef`](../type-aliases/AccountRef.md)

***

### type?

> `optional` **type**: [`NoteVisibility`](../type-aliases/NoteVisibility.md)

***

### waitForConfirmation?

> `optional` **waitForConfirmation**: `boolean`

#### Inherited from

[`TransactionOptions`](TransactionOptions.md).[`waitForConfirmation`](TransactionOptions.md#waitforconfirmation)
