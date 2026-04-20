[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / MidenClient

# Class: MidenClient

## Constructors

### Constructor

> **new MidenClient**(): `MidenClient`

#### Returns

`MidenClient`

## Properties

### accounts

> `readonly` **accounts**: [`AccountsResource`](../interfaces/AccountsResource.md)

***

### compile

> `readonly` **compile**: [`CompilerResource`](CompilerResource.md)

***

### defaultProver

> `readonly` **defaultProver**: [`TransactionProver`](TransactionProver.md)

Returns the client-level default prover.

***

### keystore

> `readonly` **keystore**: [`KeystoreResource`](../interfaces/KeystoreResource.md)

***

### notes

> `readonly` **notes**: [`NotesResource`](../interfaces/NotesResource.md)

***

### settings

> `readonly` **settings**: [`SettingsResource`](../interfaces/SettingsResource.md)

***

### tags

> `readonly` **tags**: [`TagsResource`](../interfaces/TagsResource.md)

***

### transactions

> `readonly` **transactions**: [`TransactionsResource`](../interfaces/TransactionsResource.md)

## Methods

### \[asyncDispose\]()

> **\[asyncDispose\]**(): `Promise`\<`void`\>

#### Returns

`Promise`\<`void`\>

***

### \[dispose\]()

> **\[dispose\]**(): `void`

#### Returns

`void`

***

### getSyncHeight()

> **getSyncHeight**(): `Promise`\<`number`\>

Returns the current sync height.

#### Returns

`Promise`\<`number`\>

***

### lastAuthError()

> **lastAuthError**(): `unknown`

Returns the raw JS value that the most recent sign-callback invocation
threw, or `null` if the last sign call succeeded (or no call has
happened yet). Useful for recovering structured metadata (e.g. a
`reason: 'locked'` property) that the kernel-level `auth::request`
diagnostic would otherwise erase.

#### Returns

`unknown`

***

### proveBlock()

> **proveBlock**(): `void`

Advances the mock chain by one block. Only available on mock clients.

#### Returns

`void`

***

### serializeMockChain()

> **serializeMockChain**(): `Uint8Array`

Serializes the mock chain state for snapshot/restore in tests.

#### Returns

`Uint8Array`

***

### serializeMockNoteTransportNode()

> **serializeMockNoteTransportNode**(): `Uint8Array`

Serializes the mock note transport node state.

#### Returns

`Uint8Array`

***

### storeIdentifier()

> **storeIdentifier**(): `string`

Returns the identifier of the underlying store (e.g. IndexedDB database name, file path).

#### Returns

`string`

***

### sync()

> **sync**(`options?`): `Promise`\<[`SyncSummary`](SyncSummary.md)\>

Syncs the client state with the Miden node.

#### Parameters

##### options?

###### timeout?

`number`

#### Returns

`Promise`\<[`SyncSummary`](SyncSummary.md)\>

***

### terminate()

> **terminate**(): `void`

Terminates the underlying Web Worker. After this, all method calls throw.

#### Returns

`void`

***

### usesMockChain()

> **usesMockChain**(): `boolean`

Returns true if this client uses a mock chain.

#### Returns

`boolean`

***

### waitForIdle()

> **waitForIdle**(): `Promise`\<`void`\>

Resolves once every serialized WASM call that was in flight AT THE
MOMENT `waitForIdle()` was called (execute, submit, prove, apply,
sync, or account creation) has settled. Use this from callers that
need to perform a non-WASM-side action — e.g. clearing an in-memory
auth key on wallet lock — after the kernel finishes, so its auth
callback doesn't race with the key being cleared. Does NOT wait for
calls enqueued after `waitForIdle()` returns. Returns immediately if
nothing was in flight.

#### Returns

`Promise`\<`void`\>

***

### create()

> `static` **create**(`options?`): `Promise`\<`MidenClient`\>

Creates and initializes a new MidenClient.

#### Parameters

##### options?

[`ClientOptions`](../interfaces/ClientOptions.md)

#### Returns

`Promise`\<`MidenClient`\>

***

### createDevnet()

> `static` **createDevnet**(`options?`): `Promise`\<`MidenClient`\>

Creates a client preconfigured for devnet (rpc, prover, note transport, autoSync).

#### Parameters

##### options?

[`ClientOptions`](../interfaces/ClientOptions.md)

#### Returns

`Promise`\<`MidenClient`\>

***

### createMock()

> `static` **createMock**(`options?`): `Promise`\<`MidenClient`\>

Creates a mock client for testing.

#### Parameters

##### options?

[`MockOptions`](../interfaces/MockOptions.md)

#### Returns

`Promise`\<`MidenClient`\>

***

### createTestnet()

> `static` **createTestnet**(`options?`): `Promise`\<`MidenClient`\>

Creates a client preconfigured for testnet (rpc, prover, note transport, autoSync).

#### Parameters

##### options?

[`ClientOptions`](../interfaces/ClientOptions.md)

#### Returns

`Promise`\<`MidenClient`\>
