[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / MockWebClient

# Class: MockWebClient

## Extends

- `any`

## Constructors

### Constructor

> **new MockWebClient**(): `MockWebClient`

#### Returns

`MockWebClient`

#### Inherited from

`WasmWebClient.constructor`

## Methods

### syncState()

> **syncState**(): `Promise`\<`SyncSummary`\>

Syncs the mock state and returns the resulting summary.

#### Returns

`Promise`\<`SyncSummary`\>

***

### syncStateWithTimeout()

> **syncStateWithTimeout**(`timeoutMs?`): `Promise`\<`SyncSummary`\>

Syncs the client state with the Miden node with an optional timeout.

#### Parameters

##### timeoutMs?

`number`

Optional timeout in milliseconds. If 0 or not provided, waits indefinitely.

#### Returns

`Promise`\<`SyncSummary`\>

A promise that resolves to a SyncSummary with the sync results.

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
