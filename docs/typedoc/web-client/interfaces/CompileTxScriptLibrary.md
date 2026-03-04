[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / CompileTxScriptLibrary

# Interface: CompileTxScriptLibrary

## Properties

### code

> **code**: `string`

***

### linking?

> `optional` **linking**: `"static"` \| `"dynamic"`

"static"  — copies library into the script (for off-chain libraries).
"dynamic" — links without copying (for on-chain FPI libraries). Default.

***

### namespace

> **namespace**: `string`
