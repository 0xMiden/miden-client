[**@miden-sdk/miden-wallet-adapter-base**](../README.md)

***

[@miden-sdk/miden-wallet-adapter-base](../README.md) / BaseMessageSignerWalletAdapter

# Abstract Class: BaseMessageSignerWalletAdapter\<Name\>

Minimal `EventEmitter` interface that is molded against the Node.js
`EventEmitter` interface.

## Extends

- [`BaseSignerWalletAdapter`](BaseSignerWalletAdapter.md)\<`Name`\>

## Type Parameters

### Name

`Name` *extends* `string` = `string`

## Implements

- [`MessageSignerWalletAdapter`](../type-aliases/MessageSignerWalletAdapter.md)\<`Name`\>

## Constructors

### Constructor

> **new BaseMessageSignerWalletAdapter**\<`Name`\>(): `BaseMessageSignerWalletAdapter`\<`Name`\>

#### Returns

`BaseMessageSignerWalletAdapter`\<`Name`\>

#### Inherited from

[`BaseSignerWalletAdapter`](BaseSignerWalletAdapter.md).[`constructor`](BaseSignerWalletAdapter.md#constructor)

## Properties

### address

> `abstract` **address**: `string`

#### Implementation of

`MessageSignerWalletAdapter.address`

#### Inherited from

[`BaseSignerWalletAdapter`](BaseSignerWalletAdapter.md).[`address`](BaseSignerWalletAdapter.md#address)

***

### connecting

> `abstract` **connecting**: `boolean`

#### Implementation of

`MessageSignerWalletAdapter.connecting`

#### Inherited from

[`BaseSignerWalletAdapter`](BaseSignerWalletAdapter.md).[`connecting`](BaseSignerWalletAdapter.md#connecting)

***

### icon

> `abstract` **icon**: `string`

#### Implementation of

`MessageSignerWalletAdapter.icon`

#### Inherited from

[`BaseSignerWalletAdapter`](BaseSignerWalletAdapter.md).[`icon`](BaseSignerWalletAdapter.md#icon)

***

### name

> `abstract` **name**: [`WalletName`](../type-aliases/WalletName.md)\<`Name`\>

#### Implementation of

`MessageSignerWalletAdapter.name`

#### Inherited from

[`BaseSignerWalletAdapter`](BaseSignerWalletAdapter.md).[`name`](BaseSignerWalletAdapter.md#name-1)

***

### publicKey

> `abstract` **publicKey**: `Uint8Array`\<`ArrayBufferLike`\>

#### Implementation of

`MessageSignerWalletAdapter.publicKey`

#### Inherited from

[`BaseSignerWalletAdapter`](BaseSignerWalletAdapter.md).[`publicKey`](BaseSignerWalletAdapter.md#publickey)

***

### readyState

> `abstract` **readyState**: [`WalletReadyState`](../enumerations/WalletReadyState.md)

#### Implementation of

`MessageSignerWalletAdapter.readyState`

#### Inherited from

[`BaseSignerWalletAdapter`](BaseSignerWalletAdapter.md).[`readyState`](BaseSignerWalletAdapter.md#readystate)

***

### supportedTransactionVersions

> `abstract` **supportedTransactionVersions**: `ReadonlySet`

#### Implementation of

`MessageSignerWalletAdapter.supportedTransactionVersions`

#### Inherited from

[`BaseSignerWalletAdapter`](BaseSignerWalletAdapter.md).[`supportedTransactionVersions`](BaseSignerWalletAdapter.md#supportedtransactionversions)

***

### url

> `abstract` **url**: `string`

#### Implementation of

`MessageSignerWalletAdapter.url`

#### Inherited from

[`BaseSignerWalletAdapter`](BaseSignerWalletAdapter.md).[`url`](BaseSignerWalletAdapter.md#url)

***

### prefixed

> `static` **prefixed**: `string` \| `boolean`

#### Inherited from

[`BaseSignerWalletAdapter`](BaseSignerWalletAdapter.md).[`prefixed`](BaseSignerWalletAdapter.md#prefixed)

## Accessors

### connected

#### Get Signature

> **get** **connected**(): `boolean`

##### Returns

`boolean`

#### Implementation of

`MessageSignerWalletAdapter.connected`

#### Inherited from

[`BaseSignerWalletAdapter`](BaseSignerWalletAdapter.md).[`connected`](BaseSignerWalletAdapter.md#connected)

## Methods

### addListener()

> **addListener**\<`T`\>(`event`, `fn`, `context?`): `this`

#### Type Parameters

##### T

`T` *extends* keyof [`WalletAdapterEvents`](../interfaces/WalletAdapterEvents.md)

#### Parameters

##### event

`T`

##### fn

(...`args`) => `void`

##### context?

`any`

#### Returns

`this`

#### Implementation of

`MessageSignerWalletAdapter.addListener`

#### Inherited from

[`BaseSignerWalletAdapter`](BaseSignerWalletAdapter.md).[`addListener`](BaseSignerWalletAdapter.md#addlistener)

***

### connect()

> `abstract` **connect**(`privateDataPermission`, `network`, `allowedPrivateData?`): `Promise`\<`void`\>

#### Parameters

##### privateDataPermission

[`PrivateDataPermission`](../enumerations/PrivateDataPermission.md)

##### network

[`WalletAdapterNetwork`](../enumerations/WalletAdapterNetwork.md)

##### allowedPrivateData?

[`AllowedPrivateData`](../enumerations/AllowedPrivateData.md)

#### Returns

`Promise`\<`void`\>

#### Implementation of

`MessageSignerWalletAdapter.connect`

#### Inherited from

[`BaseSignerWalletAdapter`](BaseSignerWalletAdapter.md).[`connect`](BaseSignerWalletAdapter.md#connect)

***

### disconnect()

> `abstract` **disconnect**(): `Promise`\<`void`\>

#### Returns

`Promise`\<`void`\>

#### Implementation of

`MessageSignerWalletAdapter.disconnect`

#### Inherited from

[`BaseSignerWalletAdapter`](BaseSignerWalletAdapter.md).[`disconnect`](BaseSignerWalletAdapter.md#disconnect)

***

### emit()

> **emit**\<`T`\>(`event`, ...`args`): `boolean`

Calls each of the listeners registered for a given event.

#### Type Parameters

##### T

`T` *extends* keyof [`WalletAdapterEvents`](../interfaces/WalletAdapterEvents.md)

#### Parameters

##### event

`T`

##### args

...[`ArgumentMap`](../@miden-sdk/namespaces/EventEmitter/type-aliases/ArgumentMap.md)\<[`WalletAdapterEvents`](../interfaces/WalletAdapterEvents.md)\>\[`Extract`\<`T`, keyof [`WalletAdapterEvents`](../interfaces/WalletAdapterEvents.md)\>\]

#### Returns

`boolean`

#### Implementation of

`MessageSignerWalletAdapter.emit`

#### Inherited from

[`BaseSignerWalletAdapter`](BaseSignerWalletAdapter.md).[`emit`](BaseSignerWalletAdapter.md#emit)

***

### eventNames()

> **eventNames**(): keyof [`WalletAdapterEvents`](../interfaces/WalletAdapterEvents.md)[]

Return an array listing the events for which the emitter has registered
listeners.

#### Returns

keyof [`WalletAdapterEvents`](../interfaces/WalletAdapterEvents.md)[]

#### Implementation of

`MessageSignerWalletAdapter.eventNames`

#### Inherited from

[`BaseSignerWalletAdapter`](BaseSignerWalletAdapter.md).[`eventNames`](BaseSignerWalletAdapter.md#eventnames)

***

### importPrivateNote()

> `abstract` **importPrivateNote**(`note`): `Promise`\<`string`\>

#### Parameters

##### note

`Uint8Array`

#### Returns

`Promise`\<`string`\>

#### Implementation of

`MessageSignerWalletAdapter.importPrivateNote`

***

### listenerCount()

> **listenerCount**(`event`): `number`

Return the number of listeners listening to a given event.

#### Parameters

##### event

keyof [`WalletAdapterEvents`](../interfaces/WalletAdapterEvents.md)

#### Returns

`number`

#### Implementation of

`MessageSignerWalletAdapter.listenerCount`

#### Inherited from

[`BaseSignerWalletAdapter`](BaseSignerWalletAdapter.md).[`listenerCount`](BaseSignerWalletAdapter.md#listenercount)

***

### listeners()

> **listeners**\<`T`\>(`event`): (...`args`) => `void`[]

Return the listeners registered for a given event.

#### Type Parameters

##### T

`T` *extends* keyof [`WalletAdapterEvents`](../interfaces/WalletAdapterEvents.md)

#### Parameters

##### event

`T`

#### Returns

(...`args`) => `void`[]

#### Implementation of

`MessageSignerWalletAdapter.listeners`

#### Inherited from

[`BaseSignerWalletAdapter`](BaseSignerWalletAdapter.md).[`listeners`](BaseSignerWalletAdapter.md#listeners)

***

### off()

> **off**\<`T`\>(`event`, `fn?`, `context?`, `once?`): `this`

#### Type Parameters

##### T

`T` *extends* keyof [`WalletAdapterEvents`](../interfaces/WalletAdapterEvents.md)

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

#### Implementation of

`MessageSignerWalletAdapter.off`

#### Inherited from

[`BaseSignerWalletAdapter`](BaseSignerWalletAdapter.md).[`off`](BaseSignerWalletAdapter.md#off)

***

### on()

> **on**\<`T`\>(`event`, `fn`, `context?`): `this`

Add a listener for a given event.

#### Type Parameters

##### T

`T` *extends* keyof [`WalletAdapterEvents`](../interfaces/WalletAdapterEvents.md)

#### Parameters

##### event

`T`

##### fn

(...`args`) => `void`

##### context?

`any`

#### Returns

`this`

#### Implementation of

`MessageSignerWalletAdapter.on`

#### Inherited from

[`BaseSignerWalletAdapter`](BaseSignerWalletAdapter.md).[`on`](BaseSignerWalletAdapter.md#on)

***

### once()

> **once**\<`T`\>(`event`, `fn`, `context?`): `this`

Add a one-time listener for a given event.

#### Type Parameters

##### T

`T` *extends* keyof [`WalletAdapterEvents`](../interfaces/WalletAdapterEvents.md)

#### Parameters

##### event

`T`

##### fn

(...`args`) => `void`

##### context?

`any`

#### Returns

`this`

#### Implementation of

`MessageSignerWalletAdapter.once`

#### Inherited from

[`BaseSignerWalletAdapter`](BaseSignerWalletAdapter.md).[`once`](BaseSignerWalletAdapter.md#once)

***

### removeAllListeners()

> **removeAllListeners**(`event?`): `this`

Remove all listeners, or those of the specified event.

#### Parameters

##### event?

keyof [`WalletAdapterEvents`](../interfaces/WalletAdapterEvents.md)

#### Returns

`this`

#### Implementation of

`MessageSignerWalletAdapter.removeAllListeners`

#### Inherited from

[`BaseSignerWalletAdapter`](BaseSignerWalletAdapter.md).[`removeAllListeners`](BaseSignerWalletAdapter.md#removealllisteners)

***

### removeListener()

> **removeListener**\<`T`\>(`event`, `fn?`, `context?`, `once?`): `this`

Remove the listeners of a given event.

#### Type Parameters

##### T

`T` *extends* keyof [`WalletAdapterEvents`](../interfaces/WalletAdapterEvents.md)

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

#### Implementation of

`MessageSignerWalletAdapter.removeListener`

#### Inherited from

[`BaseSignerWalletAdapter`](BaseSignerWalletAdapter.md).[`removeListener`](BaseSignerWalletAdapter.md#removelistener)

***

### requestAssets()

> `abstract` **requestAssets**(): `Promise`\<[`Asset`](../interfaces/Asset.md)[]\>

#### Returns

`Promise`\<[`Asset`](../interfaces/Asset.md)[]\>

#### Implementation of

`MessageSignerWalletAdapter.requestAssets`

***

### requestConsumableNotes()

> `abstract` **requestConsumableNotes**(): `Promise`\<[`InputNoteDetails`](../type-aliases/InputNoteDetails.md)[]\>

#### Returns

`Promise`\<[`InputNoteDetails`](../type-aliases/InputNoteDetails.md)[]\>

#### Implementation of

`MessageSignerWalletAdapter.requestConsumableNotes`

***

### requestConsume()

> `abstract` **requestConsume**(`transaction`): `Promise`\<`string`\>

#### Parameters

##### transaction

[`MidenConsumeTransaction`](../interfaces/MidenConsumeTransaction.md)

#### Returns

`Promise`\<`string`\>

#### Implementation of

`MessageSignerWalletAdapter.requestConsume`

***

### requestPrivateNotes()

> `abstract` **requestPrivateNotes**(`noteFilterType`, `noteIds?`): `Promise`\<[`InputNoteDetails`](../type-aliases/InputNoteDetails.md)[]\>

#### Parameters

##### noteFilterType

`NoteFilterTypes`

##### noteIds?

`string`[]

#### Returns

`Promise`\<[`InputNoteDetails`](../type-aliases/InputNoteDetails.md)[]\>

#### Implementation of

`MessageSignerWalletAdapter.requestPrivateNotes`

***

### requestSend()

> `abstract` **requestSend**(`transaction`): `Promise`\<`string`\>

#### Parameters

##### transaction

[`MidenSendTransaction`](../interfaces/MidenSendTransaction.md)

#### Returns

`Promise`\<`string`\>

#### Implementation of

`MessageSignerWalletAdapter.requestSend`

***

### requestTransaction()

> `abstract` **requestTransaction**(`transaction`): `Promise`\<`string`\>

#### Parameters

##### transaction

[`MidenTransaction`](../interfaces/MidenTransaction.md)

#### Returns

`Promise`\<`string`\>

#### Implementation of

`MessageSignerWalletAdapter.requestTransaction`

***

### signBytes()

> `abstract` **signBytes**(`data`, `kind`): `Promise`\<`Uint8Array`\<`ArrayBufferLike`\>\>

#### Parameters

##### data

`Uint8Array`

##### kind

[`SignKind`](../type-aliases/SignKind.md)

#### Returns

`Promise`\<`Uint8Array`\<`ArrayBufferLike`\>\>

#### Implementation of

`MessageSignerWalletAdapter.signBytes`

***

### waitForTransaction()

> `abstract` **waitForTransaction**(`txId`, `timeout?`): `Promise`\<[`TransactionOutput`](../interfaces/TransactionOutput.md)\>

#### Parameters

##### txId

`string`

##### timeout?

`number`

#### Returns

`Promise`\<[`TransactionOutput`](../interfaces/TransactionOutput.md)\>

#### Implementation of

`MessageSignerWalletAdapter.waitForTransaction`
