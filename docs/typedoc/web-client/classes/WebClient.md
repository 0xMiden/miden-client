[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / WebClient

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

[`SecretKey`](SecretKey.md)

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

### createClient()

> **createClient**(`node_url?`, `node_note_transport_url?`, `seed?`): `Promise`\<`any`\>

Creates a new client with the given node URL and optional seed.
If `node_url` is `None`, it defaults to the testnet endpoint.

#### Parameters

##### node\_url?

`string`

##### node\_note\_transport\_url?

`string`

##### seed?

`Uint8Array`

#### Returns

`Promise`\<`any`\>

***

### createClientWithExternalKeystore()

> **createClientWithExternalKeystore**(`node_url?`, `node_note_transport_url?`, `seed?`, `get_key_cb?`, `insert_key_cb?`, `sign_cb?`): `Promise`\<`any`\>

Creates a new client with the given node URL, optional seed, and external keystore
callbacks. If `node_url` is `None`, it defaults to the testnet endpoint.

#### Parameters

##### node\_url?

`string`

##### node\_note\_transport\_url?

`string`

##### seed?

`Uint8Array`

##### get\_key\_cb?

`Function`

##### insert\_key\_cb?

`Function`

##### sign\_cb?

`Function`

#### Returns

`Promise`\<`any`\>

***

### createMockClient()

> **createMockClient**(`seed?`, `serialized_mock_chain?`, `serialized_mock_note_transport_node?`): `Promise`\<`any`\>

Creates a new client with a mock RPC API. Useful for testing purposes and proof-of-concept
applications as it uses a mock chain that simulates the behavior of a real node.

#### Parameters

##### seed?

`Uint8Array`

##### serialized\_mock\_chain?

`Uint8Array`

##### serialized\_mock\_note\_transport\_node?

`Uint8Array`

#### Returns

`Promise`\<`any`\>

***

### createScriptBuilder()

> **createScriptBuilder**(): [`ScriptBuilder`](ScriptBuilder.md)

#### Returns

[`ScriptBuilder`](ScriptBuilder.md)

***

### exportAccountFile()

> **exportAccountFile**(`account_id`): `Promise`\<[`AccountFile`](AccountFile.md)\>

#### Parameters

##### account\_id

[`AccountId`](AccountId.md)

#### Returns

`Promise`\<[`AccountFile`](AccountFile.md)\>

***

### exportNoteFile()

> **exportNoteFile**(`note_id`, `export_type`): `Promise`\<[`NoteFile`](NoteFile.md)\>

#### Parameters

##### note\_id

`string`

##### export\_type

`string`

#### Returns

`Promise`\<[`NoteFile`](NoteFile.md)\>

***

### exportStore()

> **exportStore**(): `Promise`\<`any`\>

Retrieves the entire underlying web store and returns it as a `JsValue`

Meant to be used in conjunction with the `force_import_store` method

#### Returns

`Promise`\<`any`\>

***

### fetchAllPrivateNotes()

> **fetchAllPrivateNotes**(): `Promise`\<`void`\>

Fetch all private notes from the note transport layer

Fetches all notes stored in the transport layer, with no pagination.
Prefer using [`WebClient::fetch_private_notes`] for a more efficient, on-going,
fetching mechanism.

#### Returns

`Promise`\<`void`\>

***

### fetchPrivateNotes()

> **fetchPrivateNotes**(): `Promise`\<`void`\>

Fetch private notes from the note transport layer

Uses an internal pagination mechanism to avoid fetching duplicate notes.

#### Returns

`Promise`\<`void`\>

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

> **getAccount**(`account_id`): `Promise`\<[`Account`](Account.md)\>

#### Parameters

##### account\_id

[`AccountId`](AccountId.md)

#### Returns

`Promise`\<[`Account`](Account.md)\>

***

### getAccountAuthByPubKey()

> **getAccountAuthByPubKey**(`pub_key`): `Promise`\<[`SecretKey`](SecretKey.md)\>

#### Parameters

##### pub\_key

[`Word`](Word.md)

#### Returns

`Promise`\<[`SecretKey`](SecretKey.md)\>

***

### getAccounts()

> **getAccounts**(): `Promise`\<[`AccountHeader`](AccountHeader.md)[]\>

#### Returns

`Promise`\<[`AccountHeader`](AccountHeader.md)[]\>

***

### getConsumableNotes()

> **getConsumableNotes**(`account_id?`): `Promise`\<[`ConsumableNoteRecord`](ConsumableNoteRecord.md)[]\>

#### Parameters

##### account\_id?

[`AccountId`](AccountId.md)

#### Returns

`Promise`\<[`ConsumableNoteRecord`](ConsumableNoteRecord.md)[]\>

***

### getInputNote()

> **getInputNote**(`note_id`): `Promise`\<[`InputNoteRecord`](InputNoteRecord.md)\>

#### Parameters

##### note\_id

`string`

#### Returns

`Promise`\<[`InputNoteRecord`](InputNoteRecord.md)\>

***

### getInputNotes()

> **getInputNotes**(`filter`): `Promise`\<[`InputNoteRecord`](InputNoteRecord.md)[]\>

#### Parameters

##### filter

[`NoteFilter`](NoteFilter.md)

#### Returns

`Promise`\<[`InputNoteRecord`](InputNoteRecord.md)[]\>

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

[`NoteFilter`](NoteFilter.md)

#### Returns

`Promise`\<`any`\>

***

### getSetting()

> **getSetting**(`key`): `Promise`\<`any`\>

Retrieves the setting value for `key`, or `None` if it hasn’t been set.

#### Parameters

##### key

`string`

#### Returns

`Promise`\<`any`\>

***

### getSyncHeight()

> **getSyncHeight**(): `Promise`\<`number`\>

#### Returns

`Promise`\<`number`\>

***

### getTransactions()

> **getTransactions**(`transaction_filter`): `Promise`\<[`TransactionRecord`](TransactionRecord.md)[]\>

#### Parameters

##### transaction\_filter

[`TransactionFilter`](TransactionFilter.md)

#### Returns

`Promise`\<[`TransactionRecord`](TransactionRecord.md)[]\>

***

### importAccountById()

> **importAccountById**(`account_id`): `Promise`\<`any`\>

#### Parameters

##### account\_id

[`AccountId`](AccountId.md)

#### Returns

`Promise`\<`any`\>

***

### importAccountFile()

> **importAccountFile**(`account_file`): `Promise`\<`any`\>

#### Parameters

##### account\_file

[`AccountFile`](AccountFile.md)

#### Returns

`Promise`\<`any`\>

***

### importNoteFile()

> **importNoteFile**(`note_file`): `Promise`\<[`NoteId`](NoteId.md)\>

#### Parameters

##### note\_file

[`NoteFile`](NoteFile.md)

#### Returns

`Promise`\<[`NoteId`](NoteId.md)\>

***

### importPublicAccountFromSeed()

> **importPublicAccountFromSeed**(`init_seed`, `mutable`): `Promise`\<[`Account`](Account.md)\>

#### Parameters

##### init\_seed

`Uint8Array`

##### mutable

`boolean`

#### Returns

`Promise`\<[`Account`](Account.md)\>

***

### insertAccountAddress()

> **insertAccountAddress**(`account_id`, `address`): `Promise`\<`void`\>

#### Parameters

##### account\_id

[`AccountId`](AccountId.md)

##### address

[`Address`](Address.md)

#### Returns

`Promise`\<`void`\>

***

### listSettingKeys()

> **listSettingKeys**(): `Promise`\<`string`[]\>

Returns all the existing setting keys from the store.

#### Returns

`Promise`\<`string`[]\>

***

### listTags()

> **listTags**(): `Promise`\<`any`\>

#### Returns

`Promise`\<`any`\>

***

### newAccount()

> **newAccount**(`account`, `overwrite`): `Promise`\<`void`\>

#### Parameters

##### account

[`Account`](Account.md)

##### overwrite

`boolean`

#### Returns

`Promise`\<`void`\>

***

### newConsumeTransactionRequest()

> **newConsumeTransactionRequest**(`list_of_note_ids`): [`TransactionRequest`](TransactionRequest.md)

#### Parameters

##### list\_of\_note\_ids

`string`[]

#### Returns

[`TransactionRequest`](TransactionRequest.md)

***

### newFaucet()

> **newFaucet**(`storage_mode`, `non_fungible`, `token_symbol`, `decimals`, `max_supply`): `Promise`\<[`Account`](Account.md)\>

#### Parameters

##### storage\_mode

[`AccountStorageMode`](AccountStorageMode.md)

##### non\_fungible

`boolean`

##### token\_symbol

`string`

##### decimals

`number`

##### max\_supply

`bigint`

#### Returns

`Promise`\<[`Account`](Account.md)\>

***

### newMintTransactionRequest()

> **newMintTransactionRequest**(`target_account_id`, `faucet_id`, `note_type`, `amount`): [`TransactionRequest`](TransactionRequest.md)

#### Parameters

##### target\_account\_id

[`AccountId`](AccountId.md)

##### faucet\_id

[`AccountId`](AccountId.md)

##### note\_type

[`NoteType`](../enumerations/NoteType.md)

##### amount

`bigint`

#### Returns

[`TransactionRequest`](TransactionRequest.md)

***

### newSendTransactionRequest()

> **newSendTransactionRequest**(`sender_account_id`, `target_account_id`, `faucet_id`, `note_type`, `amount`, `recall_height?`, `timelock_height?`): [`TransactionRequest`](TransactionRequest.md)

#### Parameters

##### sender\_account\_id

[`AccountId`](AccountId.md)

##### target\_account\_id

[`AccountId`](AccountId.md)

##### faucet\_id

[`AccountId`](AccountId.md)

##### note\_type

[`NoteType`](../enumerations/NoteType.md)

##### amount

`bigint`

##### recall\_height?

`number`

##### timelock\_height?

`number`

#### Returns

[`TransactionRequest`](TransactionRequest.md)

***

### newSwapTransactionRequest()

> **newSwapTransactionRequest**(`sender_account_id`, `offered_asset_faucet_id`, `offered_asset_amount`, `requested_asset_faucet_id`, `requested_asset_amount`, `note_type`, `payback_note_type`): [`TransactionRequest`](TransactionRequest.md)

#### Parameters

##### sender\_account\_id

[`AccountId`](AccountId.md)

##### offered\_asset\_faucet\_id

[`AccountId`](AccountId.md)

##### offered\_asset\_amount

`bigint`

##### requested\_asset\_faucet\_id

[`AccountId`](AccountId.md)

##### requested\_asset\_amount

`bigint`

##### note\_type

[`NoteType`](../enumerations/NoteType.md)

##### payback\_note\_type

[`NoteType`](../enumerations/NoteType.md)

#### Returns

[`TransactionRequest`](TransactionRequest.md)

***

### newTransaction()

> **newTransaction**(`account_id`, `transaction_request`): `Promise`\<[`TransactionResult`](TransactionResult.md)\>

#### Parameters

##### account\_id

[`AccountId`](AccountId.md)

##### transaction\_request

[`TransactionRequest`](TransactionRequest.md)

#### Returns

`Promise`\<[`TransactionResult`](TransactionResult.md)\>

***

### newWallet()

> **newWallet**(`storage_mode`, `mutable`, `init_seed?`): `Promise`\<[`Account`](Account.md)\>

#### Parameters

##### storage\_mode

[`AccountStorageMode`](AccountStorageMode.md)

##### mutable

`boolean`

##### init\_seed?

`Uint8Array`

#### Returns

`Promise`\<[`Account`](Account.md)\>

***

### proveBlock()

> **proveBlock**(): `void`

#### Returns

`void`

***

### removeAccountAddress()

> **removeAccountAddress**(`account_id`, `address`): `Promise`\<`void`\>

#### Parameters

##### account\_id

[`AccountId`](AccountId.md)

##### address

[`Address`](Address.md)

#### Returns

`Promise`\<`void`\>

***

### removeSetting()

> **removeSetting**(`key`): `Promise`\<`void`\>

Deletes a setting key-value from the store.

#### Parameters

##### key

`string`

#### Returns

`Promise`\<`void`\>

***

### removeTag()

> **removeTag**(`tag`): `Promise`\<`void`\>

#### Parameters

##### tag

`string`

#### Returns

`Promise`\<`void`\>

***

### sendPrivateNote()

> **sendPrivateNote**(`note`, `address`): `Promise`\<`void`\>

Send a private note via the note transport layer

#### Parameters

##### note

[`Note`](Note.md)

##### address

[`Address`](Address.md)

#### Returns

`Promise`\<`void`\>

***

### serializeMockChain()

> **serializeMockChain**(): `Uint8Array`

Returns the inner serialized mock chain if it exists.

#### Returns

`Uint8Array`

***

### serializeMockNoteTransportNode()

> **serializeMockNoteTransportNode**(): `Uint8Array`

Returns the inner serialized mock note transport node if it exists.

#### Returns

`Uint8Array`

***

### setSetting()

> **setSetting**(`key`, `value`): `Promise`\<`void`\>

Sets a setting key-value in the store. It can then be retrieved using `get_setting`.

#### Parameters

##### key

`string`

##### value

`any`

#### Returns

`Promise`\<`void`\>

***

### submitTransaction()

> **submitTransaction**(`transaction_result`, `prover?`): `Promise`\<`void`\>

#### Parameters

##### transaction\_result

[`TransactionResult`](TransactionResult.md)

##### prover?

[`TransactionProver`](TransactionProver.md)

#### Returns

`Promise`\<`void`\>

***

### syncState()

> **syncState**(): `Promise`\<[`SyncSummary`](SyncSummary.md)\>

#### Returns

`Promise`\<[`SyncSummary`](SyncSummary.md)\>

***

### testingApplyTransaction()

> **testingApplyTransaction**(`tx_result`): `Promise`\<`void`\>

#### Parameters

##### tx\_result

[`TransactionResult`](TransactionResult.md)

#### Returns

`Promise`\<`void`\>

***

### usesMockChain()

> **usesMockChain**(): `boolean`

#### Returns

`boolean`

***

### buildSwapTag()

> `static` **buildSwapTag**(`note_type`, `offered_asset_faucet_id`, `offered_asset_amount`, `requested_asset_faucet_id`, `requested_asset_amount`): [`NoteTag`](NoteTag.md)

#### Parameters

##### note\_type

[`NoteType`](../enumerations/NoteType.md)

##### offered\_asset\_faucet\_id

[`AccountId`](AccountId.md)

##### offered\_asset\_amount

`bigint`

##### requested\_asset\_faucet\_id

[`AccountId`](AccountId.md)

##### requested\_asset\_amount

`bigint`

#### Returns

[`NoteTag`](NoteTag.md)
