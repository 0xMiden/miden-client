[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / TransactionOptions

# Interface: TransactionOptions

## Extended by

- [`SendOptions`](SendOptions.md)
- [`MintOptions`](MintOptions.md)
- [`ConsumeOptions`](ConsumeOptions.md)
- [`ConsumeAllOptions`](ConsumeAllOptions.md)
- [`SwapOptions`](SwapOptions.md)
- [`MintAndConsumeOptions`](MintAndConsumeOptions.md)

## Properties

### prover?

> `optional` **prover**: [`TransactionProver`](../classes/TransactionProver.md)

Override default prover.

***

### timeout?

> `optional` **timeout**: `number`

Timeout in ms (default: 60_000).

***

### waitForConfirmation?

> `optional` **waitForConfirmation**: `boolean`
