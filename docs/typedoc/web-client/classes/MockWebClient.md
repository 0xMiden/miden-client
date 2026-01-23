[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / MockWebClient

# Class: MockWebClient

## Extends

- [`WebClient`](WebClient.md)

## Constructors

### Constructor

> **new MockWebClient**(): `MockWebClient`

#### Returns

`MockWebClient`

#### Inherited from

[`WebClient`](WebClient.md).[`constructor`](WebClient.md#constructor)

## Methods

### \[dispose\]()

> **\[dispose\]**(): `void`

#### Returns

`void`

#### Inherited from

[`WebClient`](WebClient.md).[`[dispose]`](WebClient.md#dispose)

***

### addAccountSecretKeyToWebStore()

> **addAccountSecretKeyToWebStore**(`account_id`, `secret_key`): `Promise`\<`void`\>

#### Parameters

##### account\_id

[`AccountId`](AccountId.md)

##### secret\_key

[`AuthSecretKey`](AuthSecretKey.md)

#### Returns

`Promise`\<`void`\>

#### Inherited from

[`WebClient`](WebClient.md).[`addAccountSecretKeyToWebStore`](WebClient.md#addaccountsecretkeytowebstore)

***

### addTag()

> **addTag**(`tag`): `Promise`\<`void`\>

#### Parameters

##### tag

`string`

#### Returns

`Promise`\<`void`\>

#### Inherited from

[`WebClient`](WebClient.md).[`addTag`](WebClient.md#addtag)

***

### applyTransaction()

> **applyTransaction**(`transaction_result`, `submission_height`): `Promise`\<[`TransactionStoreUpdate`](TransactionStoreUpdate.md)\>

#### Parameters

##### transaction\_result

[`TransactionResult`](TransactionResult.md)

##### submission\_height

`number`

#### Returns

`Promise`\<[`TransactionStoreUpdate`](TransactionStoreUpdate.md)\>

#### Inherited from

[`WebClient`](WebClient.md).[`applyTransaction`](WebClient.md#applytransaction)

***

### createClient()

#### Call Signature

> **createClient**(`node_url?`, `node_note_transport_url?`, `seed?`, `store_name?`): `Promise`\<`any`\>

Creates a new `WebClient` instance with the specified configuration.

# Arguments
* `node_url`: The URL of the node RPC endpoint. If `None`, defaults to the testnet endpoint.
* `node_note_transport_url`: Optional URL of the note transport service.
* `seed`: Optional seed for account initialization.
* `store_name`: Optional name for the web store. If `None`, the store name defaults to
  `MidenClientDB_{network_id}`, where `network_id` is derived from the `node_url`.
  Explicitly setting this allows for creating multiple isolated clients.

##### Parameters

###### node\_url?

`string`

###### node\_note\_transport\_url?

`string`

###### seed?

`Uint8Array`

###### store\_name?

`string`

##### Returns

`Promise`\<`any`\>

##### Inherited from

[`WebClient`](WebClient.md).[`createClient`](WebClient.md#createclient)

#### Call Signature

> **createClient**(`rpcUrl?`, `noteTransportUrl?`, `seed?`, `network?`): `Promise`\<[`WebClient`](WebClient.md)\>

Factory method to create and initialize a new wrapped WebClient.

##### Parameters

###### rpcUrl?

`string`

The RPC URL (optional).

###### noteTransportUrl?

`string`

The note transport URL (optional).

###### seed?

`Uint8Array`

The seed for the account (optional).

###### network?

`string`

Optional name for the store. Setting this allows multiple clients to be used in the same browser.

##### Returns

`Promise`\<[`WebClient`](WebClient.md)\>

A promise that resolves to a fully initialized WebClient.

##### Inherited from

[`WebClient`](WebClient.md).[`createClient`](WebClient.md#createclient)

***

### createClientWithExternalKeystore()

#### Call Signature

> **createClientWithExternalKeystore**(`node_url?`, `node_note_transport_url?`, `seed?`, `store_name?`, `get_key_cb?`, `insert_key_cb?`, `sign_cb?`): `Promise`\<`any`\>

Creates a new `WebClient` instance with external keystore callbacks.

# Arguments
* `node_url`: The URL of the node RPC endpoint. If `None`, defaults to the testnet endpoint.
* `node_note_transport_url`: Optional URL of the note transport service.
* `seed`: Optional seed for account initialization.
* `store_name`: Optional name for the web store. If `None`, the store name defaults to
  `MidenClientDB_{network_id}`, where `network_id` is derived from the `node_url`.
  Explicitly setting this allows for creating multiple isolated clients.
* `get_key_cb`: Callback to retrieve the secret key bytes for a given public key.
* `insert_key_cb`: Callback to persist a secret key.
* `sign_cb`: Callback to produce serialized signature bytes for the provided inputs.

##### Parameters

###### node\_url?

`string`

###### node\_note\_transport\_url?

`string`

###### seed?

`Uint8Array`

###### store\_name?

`string`

###### get\_key\_cb?

`Function`

###### insert\_key\_cb?

`Function`

###### sign\_cb?

`Function`

##### Returns

`Promise`\<`any`\>

##### Inherited from

[`WebClient`](WebClient.md).[`createClientWithExternalKeystore`](WebClient.md#createclientwithexternalkeystore)

#### Call Signature

> **createClientWithExternalKeystore**(`rpcUrl?`, `noteTransportUrl?`, `seed?`, `storeName?`, `getKeyCb?`, `insertKeyCb?`, `signCb?`): `Promise`\<[`WebClient`](WebClient.md)\>

Factory method to create and initialize a new wrapped WebClient with a remote keystore.

##### Parameters

###### rpcUrl?

`string`

The RPC URL (optional).

###### noteTransportUrl?

`string`

The note transport URL (optional).

###### seed?

`Uint8Array`

The seed for the account (optional).

###### storeName?

`string`

Optional name for the store. Setting this allows multiple clients to be used in the same browser.

###### getKeyCb?

[`GetKeyCallback`](../type-aliases/GetKeyCallback.md)

Callback used to retrieve secret keys for a given public key.

###### insertKeyCb?

[`InsertKeyCallback`](../type-aliases/InsertKeyCallback.md)

Callback used to persist secret keys in the external store.

###### signCb?

[`SignCallback`](../type-aliases/SignCallback.md)

Callback used to create signatures for the provided inputs.

##### Returns

`Promise`\<[`WebClient`](WebClient.md)\>

A promise that resolves to a fully initialized WebClient.

##### Inherited from

[`WebClient`](WebClient.md).[`createClientWithExternalKeystore`](WebClient.md#createclientwithexternalkeystore)

***

### createCodeBuilder()

> **createCodeBuilder**(): [`CodeBuilder`](CodeBuilder.md)

#### Returns

[`CodeBuilder`](CodeBuilder.md)

#### Inherited from

[`WebClient`](WebClient.md).[`createCodeBuilder`](WebClient.md#createcodebuilder)

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

#### Inherited from

[`WebClient`](WebClient.md).[`createMockClient`](WebClient.md#createmockclient)

***

### defaultTransactionProver()

> **defaultTransactionProver**(): [`TransactionProver`](TransactionProver.md)

Returns the default transaction prover configured on the client.

#### Returns

[`TransactionProver`](TransactionProver.md)

#### Inherited from

[`WebClient`](WebClient.md).[`defaultTransactionProver`](WebClient.md#defaulttransactionprover)

***

### executeForSummary()

> **executeForSummary**(`account_id`, `transaction_request`): `Promise`\<[`TransactionSummary`](TransactionSummary.md)\>

Executes a transaction and returns the `TransactionSummary`.

If the transaction is unauthorized (auth script emits the unauthorized event),
returns the summary from the error. If the transaction succeeds, constructs
a summary from the executed transaction using the `auth_arg` from the transaction
request as the salt (or a zero salt if not provided).

# Errors
- If there is an internal failure during execution.

#### Parameters

##### account\_id

[`AccountId`](AccountId.md)

##### transaction\_request

[`TransactionRequest`](TransactionRequest.md)

#### Returns

`Promise`\<[`TransactionSummary`](TransactionSummary.md)\>

#### Inherited from

[`WebClient`](WebClient.md).[`executeForSummary`](WebClient.md#executeforsummary)

***

### executeTransaction()

> **executeTransaction**(`account_id`, `transaction_request`): `Promise`\<[`TransactionResult`](TransactionResult.md)\>

Executes a transaction specified by the request against the specified account but does not
submit it to the network nor update the local database. The returned [`TransactionResult`]
retains the execution artifacts needed to continue with the transaction lifecycle.

If the transaction utilizes foreign account data, there is a chance that the client doesn't
have the required block header in the local database. In these scenarios, a sync to
the chain tip is performed, and the required block header is retrieved.

#### Parameters

##### account\_id

[`AccountId`](AccountId.md)

##### transaction\_request

[`TransactionRequest`](TransactionRequest.md)

#### Returns

`Promise`\<[`TransactionResult`](TransactionResult.md)\>

#### Inherited from

[`WebClient`](WebClient.md).[`executeTransaction`](WebClient.md#executetransaction)

***

### exportAccountFile()

> **exportAccountFile**(`account_id`): `Promise`\<[`AccountFile`](AccountFile.md)\>

#### Parameters

##### account\_id

[`AccountId`](AccountId.md)

#### Returns

`Promise`\<[`AccountFile`](AccountFile.md)\>

#### Inherited from

[`WebClient`](WebClient.md).[`exportAccountFile`](WebClient.md#exportaccountfile)

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

#### Inherited from

[`WebClient`](WebClient.md).[`exportNoteFile`](WebClient.md#exportnotefile)

***

### exportStore()

> **exportStore**(): `Promise`\<`any`\>

Retrieves the entire underlying web store and returns it as a `JsValue`

Meant to be used in conjunction with the `force_import_store` method

#### Returns

`Promise`\<`any`\>

#### Inherited from

[`WebClient`](WebClient.md).[`exportStore`](WebClient.md#exportstore)

***

### fetchAllPrivateNotes()

> **fetchAllPrivateNotes**(): `Promise`\<`void`\>

Fetch all private notes from the note transport layer

Fetches all notes stored in the transport layer, with no pagination.
Prefer using [`WebClient::fetch_private_notes`] for a more efficient, on-going,
fetching mechanism.

#### Returns

`Promise`\<`void`\>

#### Inherited from

[`WebClient`](WebClient.md).[`fetchAllPrivateNotes`](WebClient.md#fetchallprivatenotes)

***

### fetchPrivateNotes()

> **fetchPrivateNotes**(): `Promise`\<`void`\>

Fetch private notes from the note transport layer

Uses an internal pagination mechanism to avoid fetching duplicate notes.

#### Returns

`Promise`\<`void`\>

#### Inherited from

[`WebClient`](WebClient.md).[`fetchPrivateNotes`](WebClient.md#fetchprivatenotes)

***

### forceImportStore()

> **forceImportStore**(`store_dump`, `_store_name`): `Promise`\<`any`\>

#### Parameters

##### store\_dump

`any`

##### \_store\_name

`string`

#### Returns

`Promise`\<`any`\>

#### Inherited from

[`WebClient`](WebClient.md).[`forceImportStore`](WebClient.md#forceimportstore)

***

### free()

> **free**(): `void`

#### Returns

`void`

#### Inherited from

[`WebClient`](WebClient.md).[`free`](WebClient.md#free)

***

### getAccount()

> **getAccount**(`account_id`): `Promise`\<[`Account`](Account.md)\>

#### Parameters

##### account\_id

[`AccountId`](AccountId.md)

#### Returns

`Promise`\<[`Account`](Account.md)\>

#### Inherited from

[`WebClient`](WebClient.md).[`getAccount`](WebClient.md#getaccount)

***

### getAccountAuthByPubKeyCommitment()

> **getAccountAuthByPubKeyCommitment**(`pub_key_commitment`): `Promise`\<[`AuthSecretKey`](AuthSecretKey.md)\>

Retrieves an authentication secret key from the keystore given a public key commitment.

The public key commitment should correspond to one of the keys tracked by the keystore.
Returns the associated [`AuthSecretKey`] if found, or an error if not found.

#### Parameters

##### pub\_key\_commitment

[`Word`](Word.md)

#### Returns

`Promise`\<[`AuthSecretKey`](AuthSecretKey.md)\>

#### Inherited from

[`WebClient`](WebClient.md).[`getAccountAuthByPubKeyCommitment`](WebClient.md#getaccountauthbypubkeycommitment)

***

### getAccounts()

> **getAccounts**(): `Promise`\<[`AccountHeader`](AccountHeader.md)[]\>

#### Returns

`Promise`\<[`AccountHeader`](AccountHeader.md)[]\>

#### Inherited from

[`WebClient`](WebClient.md).[`getAccounts`](WebClient.md#getaccounts)

***

### getConsumableNotes()

> **getConsumableNotes**(`account_id?`): `Promise`\<[`ConsumableNoteRecord`](ConsumableNoteRecord.md)[]\>

#### Parameters

##### account\_id?

[`AccountId`](AccountId.md)

#### Returns

`Promise`\<[`ConsumableNoteRecord`](ConsumableNoteRecord.md)[]\>

#### Inherited from

[`WebClient`](WebClient.md).[`getConsumableNotes`](WebClient.md#getconsumablenotes)

***

### getInputNote()

> **getInputNote**(`note_id`): `Promise`\<[`InputNoteRecord`](InputNoteRecord.md)\>

#### Parameters

##### note\_id

`string`

#### Returns

`Promise`\<[`InputNoteRecord`](InputNoteRecord.md)\>

#### Inherited from

[`WebClient`](WebClient.md).[`getInputNote`](WebClient.md#getinputnote)

***

### getInputNotes()

> **getInputNotes**(`filter`): `Promise`\<[`InputNoteRecord`](InputNoteRecord.md)[]\>

#### Parameters

##### filter

[`NoteFilter`](NoteFilter.md)

#### Returns

`Promise`\<[`InputNoteRecord`](InputNoteRecord.md)[]\>

#### Inherited from

[`WebClient`](WebClient.md).[`getInputNotes`](WebClient.md#getinputnotes)

***

### getOutputNote()

> **getOutputNote**(`note_id`): `Promise`\<[`OutputNoteRecord`](OutputNoteRecord.md)\>

#### Parameters

##### note\_id

`string`

#### Returns

`Promise`\<[`OutputNoteRecord`](OutputNoteRecord.md)\>

#### Inherited from

[`WebClient`](WebClient.md).[`getOutputNote`](WebClient.md#getoutputnote)

***

### getOutputNotes()

> **getOutputNotes**(`filter`): `Promise`\<[`OutputNoteRecord`](OutputNoteRecord.md)[]\>

#### Parameters

##### filter

[`NoteFilter`](NoteFilter.md)

#### Returns

`Promise`\<[`OutputNoteRecord`](OutputNoteRecord.md)[]\>

#### Inherited from

[`WebClient`](WebClient.md).[`getOutputNotes`](WebClient.md#getoutputnotes)

***

### getPublicKeyCommitmentsOfAccount()

> **getPublicKeyCommitmentsOfAccount**(`account_id`): `Promise`\<[`Word`](Word.md)[]\>

Returns all public key commitments associated with the given account ID.

These commitments can be used with [`getAccountAuthByPubKeyCommitment`]
to retrieve the corresponding secret keys from the keystore.

#### Parameters

##### account\_id

[`AccountId`](AccountId.md)

#### Returns

`Promise`\<[`Word`](Word.md)[]\>

#### Inherited from

[`WebClient`](WebClient.md).[`getPublicKeyCommitmentsOfAccount`](WebClient.md#getpublickeycommitmentsofaccount)

***

### getSetting()

> **getSetting**(`key`): `Promise`\<`any`\>

Retrieves the setting value for `key`, or `None` if it hasn’t been set.

#### Parameters

##### key

`string`

#### Returns

`Promise`\<`any`\>

#### Inherited from

[`WebClient`](WebClient.md).[`getSetting`](WebClient.md#getsetting)

***

### getSyncHeight()

> **getSyncHeight**(): `Promise`\<`number`\>

#### Returns

`Promise`\<`number`\>

#### Inherited from

[`WebClient`](WebClient.md).[`getSyncHeight`](WebClient.md#getsyncheight)

***

### getTransactions()

> **getTransactions**(`transaction_filter`): `Promise`\<[`TransactionRecord`](TransactionRecord.md)[]\>

#### Parameters

##### transaction\_filter

[`TransactionFilter`](TransactionFilter.md)

#### Returns

`Promise`\<[`TransactionRecord`](TransactionRecord.md)[]\>

#### Inherited from

[`WebClient`](WebClient.md).[`getTransactions`](WebClient.md#gettransactions)

***

### importAccountById()

> **importAccountById**(`account_id`): `Promise`\<`any`\>

#### Parameters

##### account\_id

[`AccountId`](AccountId.md)

#### Returns

`Promise`\<`any`\>

#### Inherited from

[`WebClient`](WebClient.md).[`importAccountById`](WebClient.md#importaccountbyid)

***

### importAccountFile()

> **importAccountFile**(`account_file`): `Promise`\<`any`\>

#### Parameters

##### account\_file

[`AccountFile`](AccountFile.md)

#### Returns

`Promise`\<`any`\>

#### Inherited from

[`WebClient`](WebClient.md).[`importAccountFile`](WebClient.md#importaccountfile)

***

### importNoteFile()

> **importNoteFile**(`note_file`): `Promise`\<[`NoteId`](NoteId.md)\>

#### Parameters

##### note\_file

[`NoteFile`](NoteFile.md)

#### Returns

`Promise`\<[`NoteId`](NoteId.md)\>

#### Inherited from

[`WebClient`](WebClient.md).[`importNoteFile`](WebClient.md#importnotefile)

***

### importPublicAccountFromSeed()

> **importPublicAccountFromSeed**(`init_seed`, `mutable`, `auth_scheme`): `Promise`\<[`Account`](Account.md)\>

#### Parameters

##### init\_seed

`Uint8Array`

##### mutable

`boolean`

##### auth\_scheme

[`AuthScheme`](../enumerations/AuthScheme.md)

#### Returns

`Promise`\<[`Account`](Account.md)\>

#### Inherited from

[`WebClient`](WebClient.md).[`importPublicAccountFromSeed`](WebClient.md#importpublicaccountfromseed)

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

#### Inherited from

[`WebClient`](WebClient.md).[`insertAccountAddress`](WebClient.md#insertaccountaddress)

***

### listSettingKeys()

> **listSettingKeys**(): `Promise`\<`string`[]\>

Returns all the existing setting keys from the store.

#### Returns

`Promise`\<`string`[]\>

#### Inherited from

[`WebClient`](WebClient.md).[`listSettingKeys`](WebClient.md#listsettingkeys)

***

### listTags()

> **listTags**(): `Promise`\<`any`\>

#### Returns

`Promise`\<`any`\>

#### Inherited from

[`WebClient`](WebClient.md).[`listTags`](WebClient.md#listtags)

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

#### Inherited from

[`WebClient`](WebClient.md).[`newAccount`](WebClient.md#newaccount)

***

### newConsumeTransactionRequest()

> **newConsumeTransactionRequest**(`list_of_notes`): [`TransactionRequest`](TransactionRequest.md)

#### Parameters

##### list\_of\_notes

[`Note`](Note.md)[]

#### Returns

[`TransactionRequest`](TransactionRequest.md)

#### Inherited from

[`WebClient`](WebClient.md).[`newConsumeTransactionRequest`](WebClient.md#newconsumetransactionrequest)

***

### newFaucet()

> **newFaucet**(`storage_mode`, `non_fungible`, `token_symbol`, `decimals`, `max_supply`, `auth_scheme`): `Promise`\<[`Account`](Account.md)\>

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

##### auth\_scheme

[`AuthScheme`](../enumerations/AuthScheme.md)

#### Returns

`Promise`\<[`Account`](Account.md)\>

#### Inherited from

[`WebClient`](WebClient.md).[`newFaucet`](WebClient.md#newfaucet)

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

#### Inherited from

[`WebClient`](WebClient.md).[`newMintTransactionRequest`](WebClient.md#newminttransactionrequest)

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

#### Inherited from

[`WebClient`](WebClient.md).[`newSendTransactionRequest`](WebClient.md#newsendtransactionrequest)

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

#### Inherited from

[`WebClient`](WebClient.md).[`newSwapTransactionRequest`](WebClient.md#newswaptransactionrequest)

***

### newWallet()

> **newWallet**(`storage_mode`, `mutable`, `auth_scheme`, `init_seed?`): `Promise`\<[`Account`](Account.md)\>

#### Parameters

##### storage\_mode

[`AccountStorageMode`](AccountStorageMode.md)

##### mutable

`boolean`

##### auth\_scheme

[`AuthScheme`](../enumerations/AuthScheme.md)

##### init\_seed?

`Uint8Array`

#### Returns

`Promise`\<[`Account`](Account.md)\>

#### Inherited from

[`WebClient`](WebClient.md).[`newWallet`](WebClient.md#newwallet)

***

### proveBlock()

> **proveBlock**(): `void`

#### Returns

`void`

#### Inherited from

[`WebClient`](WebClient.md).[`proveBlock`](WebClient.md#proveblock)

***

### proveTransaction()

> **proveTransaction**(`transaction_result`, `prover?`): `Promise`\<[`ProvenTransaction`](ProvenTransaction.md)\>

Generates a transaction proof using either the provided prover or the client's default
prover if none is supplied.

#### Parameters

##### transaction\_result

[`TransactionResult`](TransactionResult.md)

##### prover?

[`TransactionProver`](TransactionProver.md)

#### Returns

`Promise`\<[`ProvenTransaction`](ProvenTransaction.md)\>

#### Inherited from

[`WebClient`](WebClient.md).[`proveTransaction`](WebClient.md#provetransaction)

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

#### Inherited from

[`WebClient`](WebClient.md).[`removeAccountAddress`](WebClient.md#removeaccountaddress)

***

### removeSetting()

> **removeSetting**(`key`): `Promise`\<`void`\>

Deletes a setting key-value from the store.

#### Parameters

##### key

`string`

#### Returns

`Promise`\<`void`\>

#### Inherited from

[`WebClient`](WebClient.md).[`removeSetting`](WebClient.md#removesetting)

***

### removeTag()

> **removeTag**(`tag`): `Promise`\<`void`\>

#### Parameters

##### tag

`string`

#### Returns

`Promise`\<`void`\>

#### Inherited from

[`WebClient`](WebClient.md).[`removeTag`](WebClient.md#removetag)

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

#### Inherited from

[`WebClient`](WebClient.md).[`sendPrivateNote`](WebClient.md#sendprivatenote)

***

### serializeMockChain()

> **serializeMockChain**(): `Uint8Array`

Returns the inner serialized mock chain if it exists.

#### Returns

`Uint8Array`

#### Inherited from

[`WebClient`](WebClient.md).[`serializeMockChain`](WebClient.md#serializemockchain)

***

### serializeMockNoteTransportNode()

> **serializeMockNoteTransportNode**(): `Uint8Array`

Returns the inner serialized mock note transport node if it exists.

#### Returns

`Uint8Array`

#### Inherited from

[`WebClient`](WebClient.md).[`serializeMockNoteTransportNode`](WebClient.md#serializemocknotetransportnode)

***

### setDebugMode()

> **setDebugMode**(`enabled`): `void`

Sets the debug mode for transaction execution.

When enabled, the transaction executor will record additional information useful for
debugging (the values on the VM stack and the state of the advice provider). This is
disabled by default since it adds overhead.

Must be called before `createClient`.

#### Parameters

##### enabled

`boolean`

#### Returns

`void`

#### Inherited from

[`WebClient`](WebClient.md).[`setDebugMode`](WebClient.md#setdebugmode)

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

#### Inherited from

[`WebClient`](WebClient.md).[`setSetting`](WebClient.md#setsetting)

***

### submitNewTransaction()

> **submitNewTransaction**(`account_id`, `transaction_request`): `Promise`\<[`TransactionId`](TransactionId.md)\>

Executes a transaction specified by the request against the specified account,
proves it, submits it to the network, and updates the local database.

Uses the prover configured for this client.

If the transaction utilizes foreign account data, there is a chance that the client doesn't
have the required block header in the local database. In these scenarios, a sync to
the chain tip is performed, and the required block header is retrieved.

#### Parameters

##### account\_id

[`AccountId`](AccountId.md)

##### transaction\_request

[`TransactionRequest`](TransactionRequest.md)

#### Returns

`Promise`\<[`TransactionId`](TransactionId.md)\>

#### Inherited from

[`WebClient`](WebClient.md).[`submitNewTransaction`](WebClient.md#submitnewtransaction)

***

### submitNewTransactionWithProver()

> **submitNewTransactionWithProver**(`account_id`, `transaction_request`, `prover`): `Promise`\<[`TransactionId`](TransactionId.md)\>

Executes a transaction specified by the request against the specified account, proves it
with the user provided prover, submits it to the network, and updates the local database.

If the transaction utilizes foreign account data, there is a chance that the client doesn't
have the required block header in the local database. In these scenarios, a sync to the
chain tip is performed, and the required block header is retrieved.

#### Parameters

##### account\_id

[`AccountId`](AccountId.md)

##### transaction\_request

[`TransactionRequest`](TransactionRequest.md)

##### prover

[`TransactionProver`](TransactionProver.md)

#### Returns

`Promise`\<[`TransactionId`](TransactionId.md)\>

#### Inherited from

[`WebClient`](WebClient.md).[`submitNewTransactionWithProver`](WebClient.md#submitnewtransactionwithprover)

***

### submitProvenTransaction()

> **submitProvenTransaction**(`proven_transaction`, `transaction_result`): `Promise`\<`number`\>

#### Parameters

##### proven\_transaction

[`ProvenTransaction`](ProvenTransaction.md)

##### transaction\_result

[`TransactionResult`](TransactionResult.md)

#### Returns

`Promise`\<`number`\>

#### Inherited from

[`WebClient`](WebClient.md).[`submitProvenTransaction`](WebClient.md#submitproventransaction)

***

### syncState()

> **syncState**(): `Promise`\<[`SyncSummary`](SyncSummary.md)\>

Syncs the mock state and returns the resulting summary.

#### Returns

`Promise`\<[`SyncSummary`](SyncSummary.md)\>

#### Overrides

[`WebClient`](WebClient.md).[`syncState`](WebClient.md#syncstate)

***

### syncStateImpl()

> **syncStateImpl**(): `Promise`\<[`SyncSummary`](SyncSummary.md)\>

Internal implementation of `sync_state`.

This method performs the actual sync operation. Concurrent call coordination
is handled at the JavaScript layer using the Web Locks API.

**Note:** Do not call this method directly. Use `syncState()` from JavaScript instead,
which provides proper coordination for concurrent calls.

#### Returns

`Promise`\<[`SyncSummary`](SyncSummary.md)\>

#### Inherited from

[`WebClient`](WebClient.md).[`syncStateImpl`](WebClient.md#syncstateimpl)

***

### syncStateWithTimeout()

> **syncStateWithTimeout**(`timeoutMs?`): `Promise`\<[`SyncSummary`](SyncSummary.md)\>

Syncs the client state with the Miden node with an optional timeout.

#### Parameters

##### timeoutMs?

`number`

Optional timeout in milliseconds. If 0 or not provided, waits indefinitely.

#### Returns

`Promise`\<[`SyncSummary`](SyncSummary.md)\>

A promise that resolves to a SyncSummary with the sync results.

#### Overrides

[`WebClient`](WebClient.md).[`syncStateWithTimeout`](WebClient.md#syncstatewithtimeout)

***

### terminate()

> **terminate**(): `void`

Terminates the underlying worker.

#### Returns

`void`

#### Inherited from

[`WebClient`](WebClient.md).[`terminate`](WebClient.md#terminate)

***

### usesMockChain()

> **usesMockChain**(): `boolean`

#### Returns

`boolean`

#### Inherited from

[`WebClient`](WebClient.md).[`usesMockChain`](WebClient.md#usesmockchain)

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

#### Inherited from

[`WebClient`](WebClient.md).[`buildSwapTag`](WebClient.md#buildswaptag)

***

### createClient()

> `static` **createClient**(`serializedMockChain?`, `serializedMockNoteTransportNode?`, `seed?`): `Promise`\<`MockWebClient`\>

Factory method to create and initialize a new wrapped MockWebClient.

#### Parameters

##### serializedMockChain?

Serialized mock chain (optional).

`Uint8Array` | `ArrayBuffer`

##### serializedMockNoteTransportNode?

Serialized mock note transport node (optional).

`Uint8Array` | `ArrayBuffer`

##### seed?

`Uint8Array`

Seed for account initialization (optional).

#### Returns

`Promise`\<`MockWebClient`\>

A promise that resolves to a fully initialized MockWebClient.
