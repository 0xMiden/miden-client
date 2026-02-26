[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / TransactionOptions

# Interface: TransactionOptions

## Extended by

- [`SendOptionsAuthenticated`](SendOptionsAuthenticated.md)
- [`SendOptionsUnauthenticated`](SendOptionsUnauthenticated.md)
- [`MintOptions`](MintOptions.md)
- [`ConsumeOptions`](ConsumeOptions.md)
- [`ConsumeAllOptions`](ConsumeAllOptions.md)
- [`SwapOptions`](SwapOptions.md)
- [`ExecuteOptions`](ExecuteOptions.md)

## Properties

### prover?

> `optional` **prover**: [`TransactionProver`](../classes/TransactionProver.md)

Override default prover.

***

### timeout?

> `optional` **timeout**: `number`

Wall-clock polling timeout in milliseconds for waitFor() (default: 60_000).
This is NOT a block height. For block-height-based parameters, see
`reclaimAfter` and `timelockUntil` on SendOptions.

***

### waitForConfirmation?

> `optional` **waitForConfirmation**: `boolean`
