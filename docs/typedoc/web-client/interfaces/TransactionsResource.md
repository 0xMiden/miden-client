[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / TransactionsResource

# Interface: TransactionsResource

## Methods

### consume()

> **consume**(`options`): `Promise`\<[`TransactionId`](../classes/TransactionId.md)\>

#### Parameters

##### options

[`ConsumeOptions`](ConsumeOptions.md)

#### Returns

`Promise`\<[`TransactionId`](../classes/TransactionId.md)\>

***

### consumeAll()

> **consumeAll**(`options`): `Promise`\<[`ConsumeAllResult`](ConsumeAllResult.md)\>

#### Parameters

##### options

[`ConsumeAllOptions`](ConsumeAllOptions.md)

#### Returns

`Promise`\<[`ConsumeAllResult`](ConsumeAllResult.md)\>

***

### execute()

> **execute**(`options`): `Promise`\<[`TransactionId`](../classes/TransactionId.md)\>

#### Parameters

##### options

[`ExecuteOptions`](ExecuteOptions.md)

#### Returns

`Promise`\<[`TransactionId`](../classes/TransactionId.md)\>

***

### list()

> **list**(`query?`): `Promise`\<[`TransactionRecord`](../classes/TransactionRecord.md)[]\>

#### Parameters

##### query?

[`TransactionQuery`](../type-aliases/TransactionQuery.md)

#### Returns

`Promise`\<[`TransactionRecord`](../classes/TransactionRecord.md)[]\>

***

### mint()

> **mint**(`options`): `Promise`\<[`TransactionId`](../classes/TransactionId.md)\>

#### Parameters

##### options

[`MintOptions`](MintOptions.md)

#### Returns

`Promise`\<[`TransactionId`](../classes/TransactionId.md)\>

***

### preview()

> **preview**(`options`): `Promise`\<[`TransactionSummary`](../classes/TransactionSummary.md)\>

#### Parameters

##### options

[`PreviewOptions`](../type-aliases/PreviewOptions.md)

#### Returns

`Promise`\<[`TransactionSummary`](../classes/TransactionSummary.md)\>

***

### send()

> **send**(`options`): `Promise`\<[`TransactionId`](../classes/TransactionId.md)\>

#### Parameters

##### options

[`SendOptions`](SendOptions.md)

#### Returns

`Promise`\<[`TransactionId`](../classes/TransactionId.md)\>

***

### submit()

> **submit**(`account`, `request`, `options?`): `Promise`\<[`TransactionId`](../classes/TransactionId.md)\>

Submit a pre-built TransactionRequest.
Note: WASM requires accountId separately, so `account` is the first argument.

#### Parameters

##### account

[`AccountRef`](../type-aliases/AccountRef.md)

##### request

[`TransactionRequest`](../classes/TransactionRequest.md)

##### options?

[`TransactionOptions`](TransactionOptions.md)

#### Returns

`Promise`\<[`TransactionId`](../classes/TransactionId.md)\>

***

### swap()

> **swap**(`options`): `Promise`\<[`TransactionId`](../classes/TransactionId.md)\>

#### Parameters

##### options

[`SwapOptions`](SwapOptions.md)

#### Returns

`Promise`\<[`TransactionId`](../classes/TransactionId.md)\>

***

### waitFor()

> **waitFor**(`txId`, `options?`): `Promise`\<`void`\>

#### Parameters

##### txId

`string` | [`TransactionId`](../classes/TransactionId.md)

##### options?

[`WaitOptions`](WaitOptions.md)

#### Returns

`Promise`\<`void`\>
