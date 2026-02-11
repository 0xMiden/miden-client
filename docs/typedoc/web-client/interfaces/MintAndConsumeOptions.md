[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / MintAndConsumeOptions

# Interface: MintAndConsumeOptions

Exception to the `account` field pattern: this composed operation executes
under TWO accounts (faucet mints, `to` consumes).

## Extends

- [`TransactionOptions`](TransactionOptions.md)

## Properties

### amount

> **amount**: `number` \| `bigint`

***

### faucet

> **faucet**: [`AccountRef`](../type-aliases/AccountRef.md)

The faucet account that executes the mint.

***

### prover?

> `optional` **prover**: [`TransactionProver`](../classes/TransactionProver.md)

Override default prover.

#### Inherited from

[`TransactionOptions`](TransactionOptions.md).[`prover`](TransactionOptions.md#prover)

***

### timeout?

> `optional` **timeout**: `number`

Timeout in ms (default: 60_000).

#### Inherited from

[`TransactionOptions`](TransactionOptions.md).[`timeout`](TransactionOptions.md#timeout)

***

### to

> **to**: [`AccountRef`](../type-aliases/AccountRef.md)

The account that receives the minted note AND consumes it.

***

### type?

> `optional` **type**: [`NoteVisibility`](../type-aliases/NoteVisibility.md)

***

### waitForConfirmation?

> `optional` **waitForConfirmation**: `boolean`

#### Inherited from

[`TransactionOptions`](TransactionOptions.md).[`waitForConfirmation`](TransactionOptions.md#waitforconfirmation)
