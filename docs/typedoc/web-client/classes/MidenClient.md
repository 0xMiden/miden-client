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

### defaultProver

> `readonly` **defaultProver**: [`TransactionProver`](TransactionProver.md)

Returns the client-level default prover.

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

### exportStore()

> **exportStore**(): `Promise`\<[`StoreSnapshot`](../interfaces/StoreSnapshot.md)\>

Exports the client store as a versioned snapshot.

#### Returns

`Promise`\<[`StoreSnapshot`](../interfaces/StoreSnapshot.md)\>

***

### getSyncHeight()

> **getSyncHeight**(): `Promise`\<`number`\>

Returns the current sync height.

#### Returns

`Promise`\<`number`\>

***

### importStore()

> **importStore**(`snapshot`): `Promise`\<`void`\>

Imports a previously exported store snapshot.

#### Parameters

##### snapshot

[`StoreSnapshot`](../interfaces/StoreSnapshot.md)

#### Returns

`Promise`\<`void`\>

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

### create()

> `static` **create**(`options?`): `Promise`\<`MidenClient`\>

Creates and initializes a new MidenClient.

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

Creates a client preconfigured for testnet use. Defaults to autoSync: true.

#### Parameters

##### options?

[`ClientOptions`](../interfaces/ClientOptions.md)

#### Returns

`Promise`\<`MidenClient`\>
