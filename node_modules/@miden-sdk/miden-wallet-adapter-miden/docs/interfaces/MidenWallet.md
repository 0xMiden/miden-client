[**@miden-sdk/miden-wallet-adapter-miden**](../README.md)

***

[@miden-sdk/miden-wallet-adapter-miden](../README.md) / MidenWallet

# Interface: MidenWallet

## Extends

- `EventEmitter`\<[`MidenWalletEvents`](MidenWalletEvents.md)\>

## Properties

### address?

> `optional` **address**: `string`

***

### publicKey?

> `optional` **publicKey**: `Uint8Array`

## Methods

### addListener()

> **addListener**\<`T`\>(`event`, `fn`, `context?`): `this`

#### Type Parameters

##### T

`T` *extends* keyof [`MidenWalletEvents`](MidenWalletEvents.md)

#### Parameters

##### event

`T`

##### fn

(...`args`) => `void`

##### context?

`any`

#### Returns

`this`

#### Inherited from

`EventEmitter.addListener`

***

### connect()

> **connect**(`privateDataPermission`, `network`, `allowedPrivateData?`): `Promise`\<`void`\>

#### Parameters

##### privateDataPermission

`PrivateDataPermission`

##### network

`WalletAdapterNetwork`

##### allowedPrivateData?

`AllowedPrivateData`

#### Returns

`Promise`\<`void`\>

***

### disconnect()

> **disconnect**(): `Promise`\<`void`\>

#### Returns

`Promise`\<`void`\>

***

### emit()

> **emit**\<`T`\>(`event`, ...`args`): `boolean`

Calls each of the listeners registered for a given event.

#### Type Parameters

##### T

`T` *extends* keyof [`MidenWalletEvents`](MidenWalletEvents.md)

#### Parameters

##### event

`T`

##### args

...`ArgumentMap`\<[`MidenWalletEvents`](MidenWalletEvents.md)\>\[`Extract`\<`T`, keyof [`MidenWalletEvents`](MidenWalletEvents.md)\>\]

#### Returns

`boolean`

#### Inherited from

`EventEmitter.emit`

***

### eventNames()

> **eventNames**(): keyof [`MidenWalletEvents`](MidenWalletEvents.md)[]

Return an array listing the events for which the emitter has registered
listeners.

#### Returns

keyof [`MidenWalletEvents`](MidenWalletEvents.md)[]

#### Inherited from

`EventEmitter.eventNames`

***

### importPrivateNote()

> **importPrivateNote**(`note`): `Promise`\<\{ `noteId`: `string`; \}\>

#### Parameters

##### note

`Uint8Array`

#### Returns

`Promise`\<\{ `noteId`: `string`; \}\>

***

### listenerCount()

> **listenerCount**(`event`): `number`

Return the number of listeners listening to a given event.

#### Parameters

##### event

keyof [`MidenWalletEvents`](MidenWalletEvents.md)

#### Returns

`number`

#### Inherited from

`EventEmitter.listenerCount`

***

### listeners()

> **listeners**\<`T`\>(`event`): (...`args`) => `void`[]

Return the listeners registered for a given event.

#### Type Parameters

##### T

`T` *extends* keyof [`MidenWalletEvents`](MidenWalletEvents.md)

#### Parameters

##### event

`T`

#### Returns

(...`args`) => `void`[]

#### Inherited from

`EventEmitter.listeners`

***

### off()

> **off**\<`T`\>(`event`, `fn?`, `context?`, `once?`): `this`

#### Type Parameters

##### T

`T` *extends* keyof [`MidenWalletEvents`](MidenWalletEvents.md)

#### Parameters

##### event

`T`

##### fn?

(...`args`) => `void`

##### context?

`any`

##### once?

`boolean`

#### Returns

`this`

#### Inherited from

`EventEmitter.off`

***

### on()

> **on**\<`T`\>(`event`, `fn`, `context?`): `this`

Add a listener for a given event.

#### Type Parameters

##### T

`T` *extends* keyof [`MidenWalletEvents`](MidenWalletEvents.md)

#### Parameters

##### event

`T`

##### fn

(...`args`) => `void`

##### context?

`any`

#### Returns

`this`

#### Inherited from

`EventEmitter.on`

***

### once()

> **once**\<`T`\>(`event`, `fn`, `context?`): `this`

Add a one-time listener for a given event.

#### Type Parameters

##### T

`T` *extends* keyof [`MidenWalletEvents`](MidenWalletEvents.md)

#### Parameters

##### event

`T`

##### fn

(...`args`) => `void`

##### context?

`any`

#### Returns

`this`

#### Inherited from

`EventEmitter.once`

***

### removeAllListeners()

> **removeAllListeners**(`event?`): `this`

Remove all listeners, or those of the specified event.

#### Parameters

##### event?

keyof [`MidenWalletEvents`](MidenWalletEvents.md)

#### Returns

`this`

#### Inherited from

`EventEmitter.removeAllListeners`

***

### removeListener()

> **removeListener**\<`T`\>(`event`, `fn?`, `context?`, `once?`): `this`

Remove the listeners of a given event.

#### Type Parameters

##### T

`T` *extends* keyof [`MidenWalletEvents`](MidenWalletEvents.md)

#### Parameters

##### event

`T`

##### fn?

(...`args`) => `void`

##### context?

`any`

##### once?

`boolean`

#### Returns

`this`

#### Inherited from

`EventEmitter.removeListener`

***

### requestAssets()

> **requestAssets**(): `Promise`\<\{ `assets`: `Asset`[]; \}\>

#### Returns

`Promise`\<\{ `assets`: `Asset`[]; \}\>

***

### requestConsumableNotes()

> **requestConsumableNotes**(): `Promise`\<\{ `consumableNotes`: `InputNoteDetails`[]; \}\>

#### Returns

`Promise`\<\{ `consumableNotes`: `InputNoteDetails`[]; \}\>

***

### requestConsume()

> **requestConsume**(`transaction`): `Promise`\<\{ `transactionId?`: `string`; \}\>

#### Parameters

##### transaction

`MidenConsumeTransaction`

#### Returns

`Promise`\<\{ `transactionId?`: `string`; \}\>

***

### requestPrivateNotes()

> **requestPrivateNotes**(`noteFilterType`, `noteIds?`): `Promise`\<\{ `privateNotes`: `InputNoteDetails`[]; \}\>

#### Parameters

##### noteFilterType

`NoteFilterTypes`

##### noteIds?

`string`[]

#### Returns

`Promise`\<\{ `privateNotes`: `InputNoteDetails`[]; \}\>

***

### requestSend()

> **requestSend**(`transaction`): `Promise`\<\{ `transactionId?`: `string`; \}\>

#### Parameters

##### transaction

`MidenSendTransaction`

#### Returns

`Promise`\<\{ `transactionId?`: `string`; \}\>

***

### requestTransaction()

> **requestTransaction**(`transaction`): `Promise`\<\{ `transactionId?`: `string`; \}\>

#### Parameters

##### transaction

`MidenTransaction`

#### Returns

`Promise`\<\{ `transactionId?`: `string`; \}\>

***

### signBytes()

> **signBytes**(`message`, `kind`): `Promise`\<\{ `signature`: `Uint8Array`; \}\>

#### Parameters

##### message

`Uint8Array`

##### kind

`SignKind`

#### Returns

`Promise`\<\{ `signature`: `Uint8Array`; \}\>

***

### waitForTransaction()

> **waitForTransaction**(`txId`, `timeout?`): `Promise`\<`WalletTransactionOutput`\>

#### Parameters

##### txId

`string`

##### timeout?

`number`

#### Returns

`Promise`\<`WalletTransactionOutput`\>
