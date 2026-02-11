[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / MockWasmWebClient

# Class: MockWasmWebClient

**`Internal`**

Low-level MockWebClient wrapper. Use MidenClient.createMock() instead.

## Extends

- [`WasmWebClient`](WasmWebClient.md)

## Constructors

### Constructor

> **new MockWasmWebClient**(): `MockWasmWebClient`

#### Returns

`MockWasmWebClient`

#### Inherited from

[`WasmWebClient`](WasmWebClient.md).[`constructor`](WasmWebClient.md#constructor)

## Methods

### \[dispose\]()

> **\[dispose\]**(): `void`

#### Returns

`void`

#### Inherited from

[`WasmWebClient`](WasmWebClient.md).[`[dispose]`](WasmWebClient.md#dispose)

***

### accountReader()

> **accountReader**(`account_id`): [`AccountReader`](AccountReader.md)

Creates a new `AccountReader` for lazy access to account data.

The `AccountReader` executes queries lazily - each method call fetches fresh data
from storage, ensuring you always see the current state.

# Arguments
* `account_id` - The ID of the account to read.

# Example
```javascript
const reader = client.accountReader(accountId);
const nonce = await reader.nonce();
const balance = await reader.getBalance(faucetId);
```

#### Parameters

##### account\_id

[`AccountId`](AccountId.md)

#### Returns

[`AccountReader`](AccountReader.md)

#### Inherited from

[`WasmWebClient`](WasmWebClient.md).[`accountReader`](WasmWebClient.md#accountreader)

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

[`WasmWebClient`](WasmWebClient.md).[`addAccountSecretKeyToWebStore`](WasmWebClient.md#addaccountsecretkeytowebstore)

***

### addTag()

> **addTag**(`tag`): `Promise`\<`void`\>

#### Parameters

##### tag

`string`

#### Returns

`Promise`\<`void`\>

#### Inherited from

[`WasmWebClient`](WasmWebClient.md).[`addTag`](WasmWebClient.md#addtag)

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

[`WasmWebClient`](WasmWebClient.md).[`applyTransaction`](WasmWebClient.md#applytransaction)

***

### createClient()

> **createClient**(`node_url?`, `node_note_transport_url?`, `seed?`, `store_name?`): `Promise`\<`any`\>

Creates a new `WebClient` instance with the specified configuration.

# Arguments
* `node_url`: The URL of the node RPC endpoint. If `None`, defaults to the testnet endpoint.
* `node_note_transport_url`: Optional URL of the note transport service.
* `seed`: Optional seed for account initialization.
* `store_name`: Optional name for the web store. If `None`, the store name defaults to
  `MidenClientDB_{network_id}`, where `network_id` is derived from the `node_url`.
  Explicitly setting this allows for creating multiple isolated clients.

#### Parameters

##### node\_url?

`string`

##### node\_note\_transport\_url?

`string`

##### seed?

`Uint8Array`

##### store\_name?

`string`

#### Returns

`Promise`\<`any`\>

#### Inherited from

[`WasmWebClient`](WasmWebClient.md).[`createClient`](WasmWebClient.md#createclient)

***

### createClientWithExternalKeystore()

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

#### Parameters

##### node\_url?

`string`

##### node\_note\_transport\_url?

`string`

##### seed?

`Uint8Array`

##### store\_name?

`string`

##### get\_key\_cb?

`Function`

##### insert\_key\_cb?

`Function`

##### sign\_cb?

`Function`

#### Returns

`Promise`\<`any`\>

#### Inherited from

[`WasmWebClient`](WasmWebClient.md).[`createClientWithExternalKeystore`](WasmWebClient.md#createclientwithexternalkeystore)

***

### createCodeBuilder()

> **createCodeBuilder**(): [`CodeBuilder`](CodeBuilder.md)

#### Returns

[`CodeBuilder`](CodeBuilder.md)

#### Inherited from

[`WasmWebClient`](WasmWebClient.md).[`createCodeBuilder`](WasmWebClient.md#createcodebuilder)

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

[`WasmWebClient`](WasmWebClient.md).[`createMockClient`](WasmWebClient.md#createmockclient)

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

[`WasmWebClient`](WasmWebClient.md).[`executeForSummary`](WasmWebClient.md#executeforsummary)

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

[`WasmWebClient`](WasmWebClient.md).[`executeTransaction`](WasmWebClient.md#executetransaction)

***

### exportAccountFile()

> **exportAccountFile**(`account_id`): `Promise`\<[`AccountFile`](AccountFile.md)\>

#### Parameters

##### account\_id

[`AccountId`](AccountId.md)

#### Returns

`Promise`\<[`AccountFile`](AccountFile.md)\>

#### Inherited from

[`WasmWebClient`](WasmWebClient.md).[`exportAccountFile`](WasmWebClient.md#exportaccountfile)

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

[`WasmWebClient`](WasmWebClient.md).[`exportNoteFile`](WasmWebClient.md#exportnotefile)

***

### exportStore()

> **exportStore**(): `Promise`\<`any`\>

Retrieves the entire underlying web store and returns it as a `JsValue`

Meant to be used in conjunction with the `force_import_store` method

#### Returns

`Promise`\<`any`\>

#### Inherited from

[`WasmWebClient`](WasmWebClient.md).[`exportStore`](WasmWebClient.md#exportstore)

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

[`WasmWebClient`](WasmWebClient.md).[`fetchAllPrivateNotes`](WasmWebClient.md#fetchallprivatenotes)

***

### fetchPrivateNotes()

> **fetchPrivateNotes**(): `Promise`\<`void`\>

Fetch private notes from the note transport layer

Uses an internal pagination mechanism to avoid fetching duplicate notes.

#### Returns

`Promise`\<`void`\>

#### Inherited from

[`WasmWebClient`](WasmWebClient.md).[`fetchPrivateNotes`](WasmWebClient.md#fetchprivatenotes)

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

[`WasmWebClient`](WasmWebClient.md).[`forceImportStore`](WasmWebClient.md#forceimportstore)

***

### free()

> **free**(): `void`

#### Returns

`void`

#### Inherited from

[`WasmWebClient`](WasmWebClient.md).[`free`](WasmWebClient.md#free)

***

### getAccount()

> **getAccount**(`account_id`): `Promise`\<[`Account`](Account.md)\>

Retrieves the full account data for the given account ID, returning `null` if not found.

This method loads the complete account state including vault, storage, and code.

#### Parameters

##### account\_id

[`AccountId`](AccountId.md)

#### Returns

`Promise`\<[`Account`](Account.md)\>

#### Inherited from

[`WasmWebClient`](WasmWebClient.md).[`getAccount`](WasmWebClient.md#getaccount)

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

[`WasmWebClient`](WasmWebClient.md).[`getAccountAuthByPubKeyCommitment`](WasmWebClient.md#getaccountauthbypubkeycommitment)

***

### getAccountByKeyCommitment()

> **getAccountByKeyCommitment**(`pub_key_commitment`): `Promise`\<[`Account`](../classes/Account.md)\>

Retrieves the full account data for the account associated with the given public key
commitment, returning `null` if no account is found.

#### Parameters

##### pub\_key\_commitment

[`Word`](../classes/Word.md)

#### Returns

`Promise`\<[`Account`](../classes/Account.md)\>

***

### getAccountCode()

> **getAccountCode**(`account_id`): `Promise`\<[`AccountCode`](AccountCode.md)\>

Retrieves the account code for a specific account.

Returns `null` if the account is not found.

#### Parameters

##### account\_id

[`AccountId`](AccountId.md)

#### Returns

`Promise`\<[`AccountCode`](AccountCode.md)\>

#### Inherited from

[`WasmWebClient`](WasmWebClient.md).[`getAccountCode`](WasmWebClient.md#getaccountcode)

***

### getAccounts()

> **getAccounts**(): `Promise`\<[`AccountHeader`](AccountHeader.md)[]\>

#### Returns

`Promise`\<[`AccountHeader`](AccountHeader.md)[]\>

#### Inherited from

[`WasmWebClient`](WasmWebClient.md).[`getAccounts`](WasmWebClient.md#getaccounts)

***

### getAccountStorage()

> **getAccountStorage**(`account_id`): `Promise`\<[`AccountStorage`](AccountStorage.md)\>

Retrieves the storage for a specific account.

To only load a specific slot, use `accountReader` instead.

#### Parameters

##### account\_id

[`AccountId`](AccountId.md)

#### Returns

`Promise`\<[`AccountStorage`](AccountStorage.md)\>

#### Inherited from

[`WasmWebClient`](WasmWebClient.md).[`getAccountStorage`](WasmWebClient.md#getaccountstorage)

***

### getAccountVault()

> **getAccountVault**(`account_id`): `Promise`\<[`AssetVault`](AssetVault.md)\>

Retrieves the asset vault for a specific account.

To check the balance for a single asset, use `accountReader` instead.

#### Parameters

##### account\_id

[`AccountId`](AccountId.md)

#### Returns

`Promise`\<[`AssetVault`](AssetVault.md)\>

#### Inherited from

[`WasmWebClient`](WasmWebClient.md).[`getAccountVault`](WasmWebClient.md#getaccountvault)

***

### getConsumableNotes()

> **getConsumableNotes**(`account_id?`): `Promise`\<[`ConsumableNoteRecord`](ConsumableNoteRecord.md)[]\>

#### Parameters

##### account\_id?

[`AccountId`](AccountId.md)

#### Returns

`Promise`\<[`ConsumableNoteRecord`](ConsumableNoteRecord.md)[]\>

#### Inherited from

[`WasmWebClient`](WasmWebClient.md).[`getConsumableNotes`](WasmWebClient.md#getconsumablenotes)

***

### getInputNote()

> **getInputNote**(`note_id`): `Promise`\<[`InputNoteRecord`](InputNoteRecord.md)\>

#### Parameters

##### note\_id

`string`

#### Returns

`Promise`\<[`InputNoteRecord`](InputNoteRecord.md)\>

#### Inherited from

[`WasmWebClient`](WasmWebClient.md).[`getInputNote`](WasmWebClient.md#getinputnote)

***

### getInputNotes()

> **getInputNotes**(`filter`): `Promise`\<[`InputNoteRecord`](InputNoteRecord.md)[]\>

#### Parameters

##### filter

[`NoteFilter`](NoteFilter.md)

#### Returns

`Promise`\<[`InputNoteRecord`](InputNoteRecord.md)[]\>

#### Inherited from

[`WasmWebClient`](WasmWebClient.md).[`getInputNotes`](WasmWebClient.md#getinputnotes)

***

### getOutputNote()

> **getOutputNote**(`note_id`): `Promise`\<[`OutputNoteRecord`](OutputNoteRecord.md)\>

#### Parameters

##### note\_id

`string`

#### Returns

`Promise`\<[`OutputNoteRecord`](OutputNoteRecord.md)\>

#### Inherited from

[`WasmWebClient`](WasmWebClient.md).[`getOutputNote`](WasmWebClient.md#getoutputnote)

***

### getOutputNotes()

> **getOutputNotes**(`filter`): `Promise`\<[`OutputNoteRecord`](OutputNoteRecord.md)[]\>

#### Parameters

##### filter

[`NoteFilter`](NoteFilter.md)

#### Returns

`Promise`\<[`OutputNoteRecord`](OutputNoteRecord.md)[]\>

#### Inherited from

[`WasmWebClient`](WasmWebClient.md).[`getOutputNotes`](WasmWebClient.md#getoutputnotes)

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

[`WasmWebClient`](WasmWebClient.md).[`getPublicKeyCommitmentsOfAccount`](WasmWebClient.md#getpublickeycommitmentsofaccount)

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

[`WasmWebClient`](WasmWebClient.md).[`getSetting`](WasmWebClient.md#getsetting)

***

### getSyncHeight()

> **getSyncHeight**(): `Promise`\<`number`\>

#### Returns

`Promise`\<`number`\>

#### Inherited from

[`WasmWebClient`](WasmWebClient.md).[`getSyncHeight`](WasmWebClient.md#getsyncheight)

***

### getTransactions()

> **getTransactions**(`transaction_filter`): `Promise`\<[`TransactionRecord`](TransactionRecord.md)[]\>

#### Parameters

##### transaction\_filter

[`TransactionFilter`](TransactionFilter.md)

#### Returns

`Promise`\<[`TransactionRecord`](TransactionRecord.md)[]\>

#### Inherited from

[`WasmWebClient`](WasmWebClient.md).[`getTransactions`](WasmWebClient.md#gettransactions)

***

### importAccountById()

> **importAccountById**(`account_id`): `Promise`\<`any`\>

#### Parameters

##### account\_id

[`AccountId`](AccountId.md)

#### Returns

`Promise`\<`any`\>

#### Inherited from

[`WasmWebClient`](WasmWebClient.md).[`importAccountById`](WasmWebClient.md#importaccountbyid)

***

### importAccountFile()

> **importAccountFile**(`account_file`): `Promise`\<`any`\>

#### Parameters

##### account\_file

[`AccountFile`](AccountFile.md)

#### Returns

`Promise`\<`any`\>

#### Inherited from

[`WasmWebClient`](WasmWebClient.md).[`importAccountFile`](WasmWebClient.md#importaccountfile)

***

### importNoteFile()

> **importNoteFile**(`note_file`): `Promise`\<[`NoteId`](NoteId.md)\>

#### Parameters

##### note\_file

[`NoteFile`](NoteFile.md)

#### Returns

`Promise`\<[`NoteId`](NoteId.md)\>

#### Inherited from

[`WasmWebClient`](WasmWebClient.md).[`importNoteFile`](WasmWebClient.md#importnotefile)

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

[`WasmWebClient`](WasmWebClient.md).[`importPublicAccountFromSeed`](WasmWebClient.md#importpublicaccountfromseed)

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

[`WasmWebClient`](WasmWebClient.md).[`insertAccountAddress`](WasmWebClient.md#insertaccountaddress)

***

### listSettingKeys()

> **listSettingKeys**(): `Promise`\<`string`[]\>

Returns all the existing setting keys from the store.

#### Returns

`Promise`\<`string`[]\>

#### Inherited from

[`WasmWebClient`](WasmWebClient.md).[`listSettingKeys`](WasmWebClient.md#listsettingkeys)

***

### listTags()

> **listTags**(): `Promise`\<`any`\>

#### Returns

`Promise`\<`any`\>

#### Inherited from

[`WasmWebClient`](WasmWebClient.md).[`listTags`](WasmWebClient.md#listtags)

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

[`WasmWebClient`](WasmWebClient.md).[`newAccount`](WasmWebClient.md#newaccount)

***

### newConsumeTransactionRequest()

> **newConsumeTransactionRequest**(`list_of_notes`): [`TransactionRequest`](TransactionRequest.md)

#### Parameters

##### list\_of\_notes

[`Note`](Note.md)[]

#### Returns

[`TransactionRequest`](TransactionRequest.md)

#### Inherited from

[`WasmWebClient`](WasmWebClient.md).[`newConsumeTransactionRequest`](WasmWebClient.md#newconsumetransactionrequest)

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

[`WasmWebClient`](WasmWebClient.md).[`newFaucet`](WasmWebClient.md#newfaucet)

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

[`WasmWebClient`](WasmWebClient.md).[`newMintTransactionRequest`](WasmWebClient.md#newminttransactionrequest)

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

[`WasmWebClient`](WasmWebClient.md).[`newSendTransactionRequest`](WasmWebClient.md#newsendtransactionrequest)

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

[`WasmWebClient`](WasmWebClient.md).[`newSwapTransactionRequest`](WasmWebClient.md#newswaptransactionrequest)

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

[`WasmWebClient`](WasmWebClient.md).[`newWallet`](WasmWebClient.md#newwallet)

***

### proveBlock()

> **proveBlock**(): `void`

#### Returns

`void`

#### Overrides

[`WasmWebClient`](WasmWebClient.md).[`proveBlock`](WasmWebClient.md#proveblock)

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

[`WasmWebClient`](WasmWebClient.md).[`proveTransaction`](WasmWebClient.md#provetransaction)

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

[`WasmWebClient`](WasmWebClient.md).[`removeAccountAddress`](WasmWebClient.md#removeaccountaddress)

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

[`WasmWebClient`](WasmWebClient.md).[`removeSetting`](WasmWebClient.md#removesetting)

***

### removeTag()

> **removeTag**(`tag`): `Promise`\<`void`\>

#### Parameters

##### tag

`string`

#### Returns

`Promise`\<`void`\>

#### Inherited from

[`WasmWebClient`](WasmWebClient.md).[`removeTag`](WasmWebClient.md#removetag)

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

[`WasmWebClient`](WasmWebClient.md).[`sendPrivateNote`](WasmWebClient.md#sendprivatenote)

***

### serializeMockChain()

> **serializeMockChain**(): `Uint8Array`

Returns the inner serialized mock chain if it exists.

#### Returns

`Uint8Array`

#### Overrides

[`WasmWebClient`](WasmWebClient.md).[`serializeMockChain`](WasmWebClient.md#serializemockchain)

***

### serializeMockNoteTransportNode()

> **serializeMockNoteTransportNode**(): `Uint8Array`

Returns the inner serialized mock note transport node if it exists.

#### Returns

`Uint8Array`

#### Overrides

[`WasmWebClient`](WasmWebClient.md).[`serializeMockNoteTransportNode`](WasmWebClient.md#serializemocknotetransportnode)

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

[`WasmWebClient`](WasmWebClient.md).[`setDebugMode`](WasmWebClient.md#setdebugmode)

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

[`WasmWebClient`](WasmWebClient.md).[`setSetting`](WasmWebClient.md#setsetting)

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

[`WasmWebClient`](WasmWebClient.md).[`submitNewTransaction`](WasmWebClient.md#submitnewtransaction)

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

[`WasmWebClient`](WasmWebClient.md).[`submitNewTransactionWithProver`](WasmWebClient.md#submitnewtransactionwithprover)

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

[`WasmWebClient`](WasmWebClient.md).[`submitProvenTransaction`](WasmWebClient.md#submitproventransaction)

***

### syncState()

> **syncState**(): `Promise`\<[`SyncSummary`](SyncSummary.md)\>

#### Returns

`Promise`\<[`SyncSummary`](SyncSummary.md)\>

#### Inherited from

[`WasmWebClient`](WasmWebClient.md).[`syncState`](WasmWebClient.md#syncstate)

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

[`WasmWebClient`](WasmWebClient.md).[`syncStateImpl`](WasmWebClient.md#syncstateimpl)

***

### syncStateWithTimeout()

> **syncStateWithTimeout**(`timeoutMs`): `Promise`\<[`SyncSummary`](SyncSummary.md)\>

#### Parameters

##### timeoutMs

`number`

#### Returns

`Promise`\<[`SyncSummary`](SyncSummary.md)\>

#### Inherited from

[`WasmWebClient`](WasmWebClient.md).[`syncStateWithTimeout`](WasmWebClient.md#syncstatewithtimeout)

***

### terminate()

> **terminate**(): `void`

#### Returns

`void`

#### Inherited from

[`WasmWebClient`](WasmWebClient.md).[`terminate`](WasmWebClient.md#terminate)

***

### usesMockChain()

> **usesMockChain**(): `boolean`

#### Returns

`boolean`

#### Inherited from

[`WasmWebClient`](WasmWebClient.md).[`usesMockChain`](WasmWebClient.md#usesmockchain)

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

[`WasmWebClient`](WasmWebClient.md).[`buildSwapTag`](WasmWebClient.md#buildswaptag)

***

### createClient()

> `static` **createClient**(`serializedMockChain?`, `serializedMockNoteTransportNode?`, `seed?`): `Promise`\<`MockWasmWebClient`\>

#### Parameters

##### serializedMockChain?

`Uint8Array`

##### serializedMockNoteTransportNode?

`Uint8Array`

##### seed?

`Uint8Array`

#### Returns

`Promise`\<`MockWasmWebClient`\>

#### Overrides

[`WasmWebClient`](WasmWebClient.md).[`createClient`](WasmWebClient.md#createclient-2)

***

### createClientWithExternalKeystore()

> `static` **createClientWithExternalKeystore**(`rpcUrl?`, `noteTransportUrl?`, `seed?`, `storeName?`, `getKeyCb?`, `insertKeyCb?`, `signCb?`): `Promise`\<[`WasmWebClient`](WasmWebClient.md)\>

#### Parameters

##### rpcUrl?

`string`

##### noteTransportUrl?

`string`

##### seed?

`Uint8Array`

##### storeName?

`string`

##### getKeyCb?

[`GetKeyCallback`](../type-aliases/GetKeyCallback.md)

##### insertKeyCb?

[`InsertKeyCallback`](../type-aliases/InsertKeyCallback.md)

##### signCb?

[`SignCallback`](../type-aliases/SignCallback.md)

#### Returns

`Promise`\<[`WasmWebClient`](WasmWebClient.md)\>

#### Inherited from

[`WasmWebClient`](WasmWebClient.md).[`createClientWithExternalKeystore`](WasmWebClient.md#createclientwithexternalkeystore-2)
