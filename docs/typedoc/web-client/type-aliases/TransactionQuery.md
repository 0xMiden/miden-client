[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / TransactionQuery

# Type Alias: TransactionQuery

> **TransactionQuery** = \{ `status`: `"uncommitted"`; \} \| \{ `ids`: (`string` \| [`TransactionId`](../classes/TransactionId.md))[]; \} \| \{ `expiredBefore`: `number`; \}

Discriminated union for transaction queries.
Mirrors the underlying WASM TransactionFilter enum.
