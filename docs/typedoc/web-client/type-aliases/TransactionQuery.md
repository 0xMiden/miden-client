[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / TransactionQuery

# Type Alias: TransactionQuery

> **TransactionQuery** = \{ `status`: `"uncommitted"`; \} \| \{ `ids`: `string`[]; \} \| \{ `expiredBefore`: `number`; \}

Discriminated union for transaction queries.
Mirrors the underlying WASM TransactionFilter enum.
