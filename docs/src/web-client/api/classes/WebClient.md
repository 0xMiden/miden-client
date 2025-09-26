---
title: WebClient
draft: true
---

[**@demox-labs/miden-sdk**](../index)

***

[@demox-labs/miden-sdk](../index) / WebClient

# Class: WebClient

## Constructors

### Constructor

> **new WebClient**(): `WebClient`

#### Returns

`WebClient`

## Methods

### addAccountSecretKeyToWebStore()

> **addAccountSecretKeyToWebStore**(`secret_key`): `Promise`\<`void`\>

#### Parameters

##### secret\_key

[`SecretKey`](SecretKey)

#### Returns

`Promise`\<`void`\>

***

### addTag()

> **addTag**(`tag`): `Promise`\<`void`\>

#### Parameters

##### tag

`string`

#### Returns

`Promise`\<`void`\>

***

### compileNoteScript()

> **compileNoteScript**(`script`): [`NoteScript`](NoteScript)

#### Parameters

##### script

`string`

#### Returns

[`NoteScript`](NoteScript)

***

### compileTxScript()

> **compileTxScript**(`script`): [`TransactionScript`](TransactionScript)

#### Parameters

##### script

`string`

#### Returns

[`TransactionScript`](TransactionScript)

***

### createClient()

> **createClient**(`node_url?`, `seed?`): `Promise`\<`any`\>

Creates a new client with the given node URL and optional seed.
If `node_url` is `None`, it defaults to the testnet endpoint.

#### Parameters

##### node\_url?

`string`

##### seed?

`Uint8Array`

#### Returns

`Promise`\<`any`\>

***

### createMockClient()

> **createMockClient**(`seed?`, `serialized_mock_chain?`): `Promise`\<`any`\>

Creates a new client with a mock RPC API. Useful for testing purposes and proof-of-concept
applications as it uses a mock chain that simulates the behavior of a real node.

#### Parameters

##### seed?

`Uint8Array`

##### serialized\_mock\_chain?

`Uint8Array`

#### Returns

`Promise`\<`any`\>

***

### exportAccountFile()

> **exportAccountFile**(`account_id`): `Promise`\<`any`\>

#### Parameters

##### account\_id

[`AccountId`](AccountId)

#### Returns

`Promise`\<`any`\>

***

### exportNoteFile()

> **exportNoteFile**(`note_id`, `export_type`): `Promise`\<`any`\>

#### Parameters

##### note\_id

`string`

##### export\_type

`string`

#### Returns

`Promise`\<`any`\>

***

### exportStore()

> **exportStore**(): `Promise`\<`any`\>

Retrieves the entire underlying web store and returns it as a `JsValue`

Meant to be used in conjunction with the `force_import_store` method

#### Returns

`Promise`\<`any`\>

***

### forceImportStore()

> **forceImportStore**(`store_dump`): `Promise`\<`any`\>

#### Parameters

##### store\_dump

`any`

#### Returns

`Promise`\<`any`\>

***

### free()

> **free**(): `void`

#### Returns

`void`

***

### getAccount()

> **getAccount**(`account_id`): `Promise`\<[`Account`](Account)\>

#### Parameters

##### account\_id

[`AccountId`](AccountId)

#### Returns

`Promise`\<[`Account`](Account)\>

***

### getAccountAuthByPubKey()

> **getAccountAuthByPubKey**(`pub_key`): `Promise`\<[`SecretKey`](SecretKey)\>

#### Parameters

##### pub\_key

[`Word`](Word)

#### Returns

`Promise`\<[`SecretKey`](SecretKey)\>

***

### getAccounts()

> **getAccounts**(): `Promise`\<[`AccountHeader`](AccountHeader)[]\>

#### Returns

`Promise`\<[`AccountHeader`](AccountHeader)[]\>

***

### getConsumableNotes()

> **getConsumableNotes**(`account_id?`): `Promise`\<[`ConsumableNoteRecord`](ConsumableNoteRecord)[]\>

#### Parameters

##### account\_id?

[`AccountId`](AccountId)

#### Returns

`Promise`\<[`ConsumableNoteRecord`](ConsumableNoteRecord)[]\>

***

### getInputNote()

> **getInputNote**(`note_id`): `Promise`\<[`InputNoteRecord`](InputNoteRecord)\>

#### Parameters

##### note\_id

`string`

#### Returns

`Promise`\<[`InputNoteRecord`](InputNoteRecord)\>

***

### getInputNotes()

> **getInputNotes**(`filter`): `Promise`\<[`InputNoteRecord`](InputNoteRecord)[]\>

#### Parameters

##### filter

[`NoteFilter`](NoteFilter)

#### Returns

`Promise`\<[`InputNoteRecord`](InputNoteRecord)[]\>

***

### getOutputNote()

> **getOutputNote**(`note_id`): `Promise`\<`any`\>

#### Parameters

##### note\_id

`string`

#### Returns

`Promise`\<`any`\>

***

### getOutputNotes()

> **getOutputNotes**(`filter`): `Promise`\<`any`\>

#### Parameters

##### filter

[`NoteFilter`](NoteFilter)

#### Returns

`Promise`\<`any`\>

***

### getSyncHeight()

> **getSyncHeight**(): `Promise`\<`number`\>

#### Returns

`Promise`\<`number`\>

***

### getTransactions()

> **getTransactions**(`transaction_filter`): `Promise`\<[`TransactionRecord`](TransactionRecord)[]\>

#### Parameters

##### transaction\_filter

[`TransactionFilter`](TransactionFilter)

#### Returns

`Promise`\<[`TransactionRecord`](TransactionRecord)[]\>

***

### importAccountById()

> **importAccountById**(`account_id`): `Promise`\<`any`\>

#### Parameters

##### account\_id

[`AccountId`](AccountId)

#### Returns

`Promise`\<`any`\>

***

### importAccountFile()

> **importAccountFile**(`account_bytes`): `Promise`\<`any`\>

#### Parameters

##### account\_bytes

`any`

#### Returns

`Promise`\<`any`\>

***

### importNoteFile()

> **importNoteFile**(`note_bytes`): `Promise`\<`any`\>

#### Parameters

##### note\_bytes

`any`

#### Returns

`Promise`\<`any`\>

***

### importPublicAccountFromSeed()

> **importPublicAccountFromSeed**(`init_seed`, `mutable`): `Promise`\<[`Account`](Account)\>

#### Parameters

##### init\_seed

`Uint8Array`

##### mutable

`boolean`

#### Returns

`Promise`\<[`Account`](Account)\>

***

### listTags()

> **listTags**(): `Promise`\<`any`\>

#### Returns

`Promise`\<`any`\>

***

### newAccount()

> **newAccount**(`account`, `account_seed`, `overwrite`): `Promise`\<`void`\>

#### Parameters

##### account

[`Account`](Account)

##### account\_seed

[`Word`](Word)

##### overwrite

`boolean`

#### Returns

`Promise`\<`void`\>

***

### newConsumeTransactionRequest()

> **newConsumeTransactionRequest**(`list_of_note_ids`): [`TransactionRequest`](TransactionRequest)

#### Parameters

##### list\_of\_note\_ids

`string`[]

#### Returns

[`TransactionRequest`](TransactionRequest)

***

### newFaucet()

> **newFaucet**(`storage_mode`, `non_fungible`, `token_symbol`, `decimals`, `max_supply`): `Promise`\<[`Account`](Account)\>

#### Parameters

##### storage\_mode

[`AccountStorageMode`](AccountStorageMode)

##### non\_fungible

`boolean`

##### token\_symbol

`string`

##### decimals

`number`

##### max\_supply

`bigint`

#### Returns

`Promise`\<[`Account`](Account)\>

***

### newMintTransactionRequest()

> **newMintTransactionRequest**(`target_account_id`, `faucet_id`, `note_type`, `amount`): [`TransactionRequest`](TransactionRequest)

#### Parameters

##### target\_account\_id

[`AccountId`](AccountId)

##### faucet\_id

[`AccountId`](AccountId)

##### note\_type

[`NoteType`](../enumerations/NoteType)

##### amount

`bigint`

#### Returns

[`TransactionRequest`](TransactionRequest)

***

### newSendTransactionRequest()

> **newSendTransactionRequest**(`sender_account_id`, `target_account_id`, `faucet_id`, `note_type`, `amount`, `recall_height?`, `timelock_height?`): [`TransactionRequest`](TransactionRequest)

#### Parameters

##### sender\_account\_id

[`AccountId`](AccountId)

##### target\_account\_id

[`AccountId`](AccountId)

##### faucet\_id

[`AccountId`](AccountId)

##### note\_type

[`NoteType`](../enumerations/NoteType)

##### amount

`bigint`

##### recall\_height?

`number`

##### timelock\_height?

`number`

#### Returns

[`TransactionRequest`](TransactionRequest)

***

### newSwapTransactionRequest()

> **newSwapTransactionRequest**(`sender_account_id`, `offered_asset_faucet_id`, `offered_asset_amount`, `requested_asset_faucet_id`, `requested_asset_amount`, `note_type`, `payback_note_type`): [`TransactionRequest`](TransactionRequest)

#### Parameters

##### sender\_account\_id

[`AccountId`](AccountId)

##### offered\_asset\_faucet\_id

[`AccountId`](AccountId)

##### offered\_asset\_amount

`bigint`

##### requested\_asset\_faucet\_id

[`AccountId`](AccountId)

##### requested\_asset\_amount

`bigint`

##### note\_type

[`NoteType`](../enumerations/NoteType)

##### payback\_note\_type

[`NoteType`](../enumerations/NoteType)

#### Returns

[`TransactionRequest`](TransactionRequest)

***

### newTransaction()

> **newTransaction**(`account_id`, `transaction_request`): `Promise`\<[`TransactionResult`](TransactionResult)\>

#### Parameters

##### account\_id

[`AccountId`](AccountId)

##### transaction\_request

[`TransactionRequest`](TransactionRequest)

#### Returns

`Promise`\<[`TransactionResult`](TransactionResult)\>

***

### newWallet()

> **newWallet**(`storage_mode`, `mutable`, `init_seed?`): `Promise`\<[`Account`](Account)\>

#### Parameters

##### storage\_mode

[`AccountStorageMode`](AccountStorageMode)

##### mutable

`boolean`

##### init\_seed?

`Uint8Array`

#### Returns

`Promise`\<[`Account`](Account)\>

***

### proveBlock()

> **proveBlock**(): `void`

#### Returns

`void`

***

### removeTag()

> **removeTag**(`tag`): `Promise`\<`void`\>

#### Parameters

##### tag

`string`

#### Returns

`Promise`\<`void`\>

***

### serializeMockChain()

> **serializeMockChain**(): `Uint8Array`

Returns the inner serialized mock chain if it exists.

#### Returns

`Uint8Array`

***

### submitTransaction()

> **submitTransaction**(`transaction_result`, `prover?`): `Promise`\<`void`\>

#### Parameters

##### transaction\_result

[`TransactionResult`](TransactionResult)

##### prover?

[`TransactionProver`](TransactionProver)

#### Returns

`Promise`\<`void`\>

***

### syncState()

> **syncState**(): `Promise`\<[`SyncSummary`](SyncSummary)\>

#### Returns

`Promise`\<[`SyncSummary`](SyncSummary)\>

***

### testingApplyTransaction()

> **testingApplyTransaction**(`tx_result`): `Promise`\<`void`\>

#### Parameters

##### tx\_result

[`TransactionResult`](TransactionResult)

#### Returns

`Promise`\<`void`\>

***

### usesMockChain()

> **usesMockChain**(): `boolean`

#### Returns

`boolean`

***

### buildSwapTag()

> `static` **buildSwapTag**(`note_type`, `offered_asset_faucet_id`, `offered_asset_amount`, `requested_asset_faucet_id`, `requested_asset_amount`): [`NoteTag`](NoteTag)

#### Parameters

##### note\_type

[`NoteType`](../enumerations/NoteType)

##### offered\_asset\_faucet\_id

[`AccountId`](AccountId)

##### offered\_asset\_amount

`bigint`

##### requested\_asset\_faucet\_id

[`AccountId`](AccountId)

##### requested\_asset\_amount

`bigint`

#### Returns

[`NoteTag`](NoteTag)
