[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / WebClient

# Class: WebClient

## Extends

- `any`

## Constructors

### Constructor

> **new WebClient**(): `WebClient`

#### Returns

`WebClient`

#### Inherited from

`WasmWebClient.constructor`

## Methods

### defaultTransactionProver()

> **defaultTransactionProver**(): `TransactionProver`

Returns the default transaction prover configured on the client.

#### Returns

`TransactionProver`

***

### syncState()

> **syncState**(): `Promise`\<`SyncSummary`\>

Syncs the client state with the Miden node.

This method coordinates concurrent calls using the Web Locks API:
- If a sync is already in progress, callers wait and receive the same result
- Cross-tab coordination ensures only one sync runs at a time per database

#### Returns

`Promise`\<`SyncSummary`\>

A promise that resolves to a SyncSummary with the sync results.

***

### syncStateWithTimeout()

> **syncStateWithTimeout**(`timeoutMs?`): `Promise`\<`SyncSummary`\>

Syncs the client state with the Miden node with an optional timeout.

This method coordinates concurrent calls using the Web Locks API:
- If a sync is already in progress, callers wait and receive the same result
- Cross-tab coordination ensures only one sync runs at a time per database
- If a timeout is specified and exceeded, the method throws an error

#### Parameters

##### timeoutMs?

`number`

Optional timeout in milliseconds. If 0 or not provided, waits indefinitely.

#### Returns

`Promise`\<`SyncSummary`\>

A promise that resolves to a SyncSummary with the sync results.

***

### terminate()

> **terminate**(): `void`

Terminates the underlying worker.

#### Returns

`void`

***

### createClient()

> `static` **createClient**(`rpcUrl?`, `noteTransportUrl?`, `seed?`, `network?`): `Promise`\<`any`\>

Factory method to create and initialize a new wrapped WebClient.

#### Parameters

##### rpcUrl?

`string`

The RPC URL (optional).

##### noteTransportUrl?

`string`

The note transport URL (optional).

##### seed?

`Uint8Array`

The seed for the account (optional).

##### network?

`string`

Optional name for the store (optional).

#### Returns

`Promise`\<`any`\>

A promise that resolves to a fully initialized WebClient.

***

### createClientWithExternalKeystore()

> `static` **createClientWithExternalKeystore**(`rpcUrl?`, `noteTransportUrl?`, `seed?`, `storeName?`, `getKeyCb?`, `insertKeyCb?`, `signCb?`): `Promise`\<`any`\>

Factory method to create and initialize a new wrapped WebClient with a remote keystore.

#### Parameters

##### rpcUrl?

`string`

The RPC URL (optional).

##### noteTransportUrl?

`string`

The note transport URL (optional).

##### seed?

`Uint8Array`

The seed for the account (optional).

##### storeName?

`string`

Optional name for the store (optional).

##### getKeyCb?

[`GetKeyCallback`](../type-aliases/GetKeyCallback.md)

Callback used to retrieve secret keys for a given public key.

##### insertKeyCb?

[`InsertKeyCallback`](../type-aliases/InsertKeyCallback.md)

Callback used to persist secret keys in the external store.

##### signCb?

[`SignCallback`](../type-aliases/SignCallback.md)

Callback used to create signatures for the provided inputs.

#### Returns

`Promise`\<`any`\>

A promise that resolves to a fully initialized WebClient.
