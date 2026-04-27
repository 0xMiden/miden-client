[**@miden-sdk/miden-wallet-adapter-base**](../README.md)

***

[@miden-sdk/miden-wallet-adapter-base](../README.md) / BaseWalletAdapter

# Abstract Class: BaseWalletAdapter\<Name\>

Minimal `EventEmitter` interface that is molded against the Node.js
`EventEmitter` interface.

## Extends

- [`EventEmitter`](EventEmitter.md)\<[`WalletAdapterEvents`](../interfaces/WalletAdapterEvents.md)\>

## Extended by

- [`BaseSignerWalletAdapter`](BaseSignerWalletAdapter.md)

## Type Parameters

### Name

`Name` *extends* `string` = `string`

## Implements

- [`WalletAdapter`](../type-aliases/WalletAdapter.md)\<`Name`\>

## Constructors

### Constructor

> **new BaseWalletAdapter**\<`Name`\>(): `BaseWalletAdapter`\<`Name`\>

#### Returns

`BaseWalletAdapter`\<`Name`\>

#### Inherited from

[`EventEmitter`](EventEmitter.md).[`constructor`](EventEmitter.md#constructor)

## Properties

### address

> `abstract` **address**: `string`

#### Implementation of

`WalletAdapter.address`

***

### connecting

> `abstract` **connecting**: `boolean`

#### Implementation of

`WalletAdapter.connecting`

***

### icon

> `abstract` **icon**: `string`

#### Implementation of

`WalletAdapter.icon`

***

### name

> `abstract` **name**: [`WalletName`](../type-aliases/WalletName.md)\<`Name`\>

#### Implementation of

`WalletAdapter.name`

***

### publicKey

> `abstract` **publicKey**: `Uint8Array`\<`ArrayBufferLike`\>

#### Implementation of

`WalletAdapter.publicKey`

***

### readyState

> `abstract` **readyState**: [`WalletReadyState`](../enumerations/WalletReadyState.md)

#### Implementation of

`WalletAdapter.readyState`

***

### supportedTransactionVersions

> `abstract` **supportedTransactionVersions**: `ReadonlySet`

#### Implementation of

`WalletAdapter.supportedTransactionVersions`

***

### url

> `abstract` **url**: `string`

#### Implementation of

`WalletAdapter.url`

***

### prefixed

> `static` **prefixed**: `string` \| `boolean`

#### Inherited from

[`EventEmitter`](EventEmitter.md).[`prefixed`](EventEmitter.md#prefixed)

## Accessors

### connected

#### Get Signature

> **get** **connected**(): `boolean`

##### Returns

`boolean`

#### Implementation of

`WalletAdapter.connected`

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

`WalletAdapter.addListener`

#### Inherited from

[`EventEmitter`](EventEmitter.md).[`addListener`](EventEmitter.md#addlistener)

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

`WalletAdapter.connect`

***

### disconnect()

> `abstract` **disconnect**(): `Promise`\<`void`\>

#### Returns

`Promise`\<`void`\>

#### Implementation of

`WalletAdapter.disconnect`

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

`WalletAdapter.emit`

#### Inherited from

[`EventEmitter`](EventEmitter.md).[`emit`](EventEmitter.md#emit)

***

### eventNames()

> **eventNames**(): keyof [`WalletAdapterEvents`](../interfaces/WalletAdapterEvents.md)[]

Return an array listing the events for which the emitter has registered
listeners.

#### Returns

keyof [`WalletAdapterEvents`](../interfaces/WalletAdapterEvents.md)[]

#### Implementation of

`WalletAdapter.eventNames`

#### Inherited from

[`EventEmitter`](EventEmitter.md).[`eventNames`](EventEmitter.md#eventnames)

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

`WalletAdapter.listenerCount`

#### Inherited from

[`EventEmitter`](EventEmitter.md).[`listenerCount`](EventEmitter.md#listenercount)

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

`WalletAdapter.listeners`

#### Inherited from

[`EventEmitter`](EventEmitter.md).[`listeners`](EventEmitter.md#listeners)

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

`WalletAdapter.off`

#### Inherited from

[`EventEmitter`](EventEmitter.md).[`off`](EventEmitter.md#off)

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

`WalletAdapter.on`

#### Inherited from

[`EventEmitter`](EventEmitter.md).[`on`](EventEmitter.md#on)

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

`WalletAdapter.once`

#### Inherited from

[`EventEmitter`](EventEmitter.md).[`once`](EventEmitter.md#once)

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

`WalletAdapter.removeAllListeners`

#### Inherited from

[`EventEmitter`](EventEmitter.md).[`removeAllListeners`](EventEmitter.md#removealllisteners)

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

`WalletAdapter.removeListener`

#### Inherited from

[`EventEmitter`](EventEmitter.md).[`removeListener`](EventEmitter.md#removelistener)
