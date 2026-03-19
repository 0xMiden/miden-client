[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / TransactionsResource

# Interface: TransactionsResource

## Methods

### consume()

> **consume**(`options`): `Promise`\<[`TransactionSubmitResult`](TransactionSubmitResult.md)\>

Consume one or more notes for an account.

#### Parameters

##### options

[`ConsumeOptions`](ConsumeOptions.md)

#### Returns

`Promise`\<[`TransactionSubmitResult`](TransactionSubmitResult.md)\>

***

### consumeAll()

> **consumeAll**(`options`): `Promise`\<[`ConsumeAllResult`](ConsumeAllResult.md)\>

Consume all available notes for an account, up to an optional limit. Returns the count of remaining notes.

#### Parameters

##### options

[`ConsumeAllOptions`](ConsumeAllOptions.md)

#### Returns

`Promise`\<[`ConsumeAllResult`](ConsumeAllResult.md)\>

***

### execute()

> **execute**(`options`): `Promise`\<[`TransactionSubmitResult`](TransactionSubmitResult.md)\>

Execute a custom transaction script with optional foreign account references.

#### Parameters

##### options

[`ExecuteOptions`](ExecuteOptions.md)

#### Returns

`Promise`\<[`TransactionSubmitResult`](TransactionSubmitResult.md)\>

***

### list()

> **list**(`query?`): `Promise`\<[`TransactionRecord`](../classes/TransactionRecord.md)[]\>

List transactions, optionally filtered by status, IDs, or expiration.

#### Parameters

##### query?

[`TransactionQuery`](../type-aliases/TransactionQuery.md)

#### Returns

`Promise`\<[`TransactionRecord`](../classes/TransactionRecord.md)[]\>

***

### mint()

> **mint**(`options`): `Promise`\<[`TransactionSubmitResult`](TransactionSubmitResult.md)\>

Mint new tokens from a faucet account.

#### Parameters

##### options

[`MintOptions`](MintOptions.md)

#### Returns

`Promise`\<[`TransactionSubmitResult`](TransactionSubmitResult.md)\>

***

### preview()

> **preview**(`options`): `Promise`\<[`TransactionSummary`](../classes/TransactionSummary.md)\>

Dry-run a transaction to preview its effects without submitting it to the network.

#### Parameters

##### options

[`PreviewOptions`](../type-aliases/PreviewOptions.md)

#### Returns

`Promise`\<[`TransactionSummary`](../classes/TransactionSummary.md)\>

***

### send()

#### Call Signature

> **send**(`options`): `Promise`\<\{ `note`: `null`; `result`: `TransactionResult`; `txId`: [`TransactionId`](../classes/TransactionId.md); \}\>

Send tokens to another account by creating a pay-to-ID note. Set `returnNote: true` to get the created note back.

##### Parameters

###### options

[`SendOptionsDefault`](SendOptionsDefault.md)

##### Returns

`Promise`\<\{ `note`: `null`; `result`: `TransactionResult`; `txId`: [`TransactionId`](../classes/TransactionId.md); \}\>

#### Call Signature

> **send**(`options`): `Promise`\<\{ `note`: [`Note`](../classes/Note.md); `result`: `TransactionResult`; `txId`: [`TransactionId`](../classes/TransactionId.md); \}\>

##### Parameters

###### options

[`SendOptionsReturnNote`](SendOptionsReturnNote.md)

##### Returns

`Promise`\<\{ `note`: [`Note`](../classes/Note.md); `result`: `TransactionResult`; `txId`: [`TransactionId`](../classes/TransactionId.md); \}\>

#### Call Signature

> **send**(`options`): `Promise`\<[`SendResult`](SendResult.md)\>

##### Parameters

###### options

[`SendOptions`](../type-aliases/SendOptions.md)

##### Returns

`Promise`\<[`SendResult`](SendResult.md)\>

***

### submit()

> **submit**(`account`, `request`, `options?`): `Promise`\<[`TransactionSubmitResult`](TransactionSubmitResult.md)\>

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

`Promise`\<[`TransactionSubmitResult`](TransactionSubmitResult.md)\>

***

### swap()

> **swap**(`options`): `Promise`\<[`TransactionSubmitResult`](TransactionSubmitResult.md)\>

Execute an atomic swap between two assets.

#### Parameters

##### options

[`SwapOptions`](SwapOptions.md)

#### Returns

`Promise`\<[`TransactionSubmitResult`](TransactionSubmitResult.md)\>

***

### waitFor()

> **waitFor**(`txId`, `options?`): `Promise`\<`void`\>

Poll until a transaction is confirmed on-chain. Throws on rejection or timeout.

#### Parameters

##### txId

`string` | [`TransactionId`](../classes/TransactionId.md)

##### options?

[`WaitOptions`](WaitOptions.md)

#### Returns

`Promise`\<`void`\>
