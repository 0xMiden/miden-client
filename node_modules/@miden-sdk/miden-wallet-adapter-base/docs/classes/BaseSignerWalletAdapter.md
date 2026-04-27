[**@miden-sdk/miden-wallet-adapter-base**](../README.md)

***

[@miden-sdk/miden-wallet-adapter-base](../README.md) / BaseSignerWalletAdapter

# Abstract Class: BaseSignerWalletAdapter\<Name\>

Minimal `EventEmitter` interface that is molded against the Node.js
`EventEmitter` interface.

## Extends

- [`BaseWalletAdapter`](BaseWalletAdapter.md)\<`Name`\>

## Extended by

- [`BaseMessageSignerWalletAdapter`](BaseMessageSignerWalletAdapter.md)

## Type Parameters

### Name

`Name` *extends* `string` = `string`

## Implements

- [`SignerWalletAdapter`](../type-aliases/SignerWalletAdapter.md)\<`Name`\>

## Constructors

### Constructor

> **new BaseSignerWalletAdapter**\<`Name`\>(): `BaseSignerWalletAdapter`\<`Name`\>

#### Returns

`BaseSignerWalletAdapter`\<`Name`\>

#### Inherited from

[`BaseWalletAdapter`](BaseWalletAdapter.md).[`constructor`](BaseWalletAdapter.md#constructor)

## Properties

### address

> `abstract` **address**: `string`

#### Implementation of

`SignerWalletAdapter.address`

#### Inherited from

[`BaseWalletAdapter`](BaseWalletAdapter.md).[`address`](BaseWalletAdapter.md#address)

***

### connecting

> `abstract` **connecting**: `boolean`

#### Implementation of

`SignerWalletAdapter.connecting`

#### Inherited from

[`BaseWalletAdapter`](BaseWalletAdapter.md).[`connecting`](BaseWalletAdapter.md#connecting)

***

### icon

> `abstract` **icon**: `string`

#### Implementation of

`SignerWalletAdapter.icon`

#### Inherited from

[`BaseWalletAdapter`](BaseWalletAdapter.md).[`icon`](BaseWalletAdapter.md#icon)

***

### name

> `abstract` **name**: [`WalletName`](../type-aliases/WalletName.md)\<`Name`\>

#### Implementation of

`SignerWalletAdapter.name`

#### Inherited from

[`BaseWalletAdapter`](BaseWalletAdapter.md).[`name`](BaseWalletAdapter.md#name-1)

***

### publicKey

> `abstract` **publicKey**: `Uint8Array`\<`ArrayBufferLike`\>

#### Implementation of

`SignerWalletAdapter.publicKey`

#### Inherited from

[`BaseWalletAdapter`](BaseWalletAdapter.md).[`publicKey`](BaseWalletAdapter.md#publickey)

***

### readyState

> `abstract` **readyState**: [`WalletReadyState`](../enumerations/WalletReadyState.md)

#### Implementation of

`SignerWalletAdapter.readyState`

#### Inherited from

[`BaseWalletAdapter`](BaseWalletAdapter.md).[`readyState`](BaseWalletAdapter.md#readystate)

***

### supportedTransactionVersions

> `abstract` **supportedTransactionVersions**: `ReadonlySet`

#### Implementation of

`SignerWalletAdapter.supportedTransactionVersions`

#### Inherited from

[`BaseWalletAdapter`](BaseWalletAdapter.md).[`supportedTransactionVersions`](BaseWalletAdapter.md#supportedtransactionversions)

***

### url

> `abstract` **url**: `string`

#### Implementation of

`SignerWalletAdapter.url`

#### Inherited from

[`BaseWalletAdapter`](BaseWalletAdapter.md).[`url`](BaseWalletAdapter.md#url)

***

### prefixed

> `static` **prefixed**: `string` \| `boolean`

#### Inherited from

[`BaseWalletAdapter`](BaseWalletAdapter.md).[`prefixed`](BaseWalletAdapter.md#prefixed)

## Accessors

### connected

#### Get Signature

> **get** **connected**(): `boolean`

##### Returns

`boolean`

#### Implementation of

`SignerWalletAdapter.connected`

#### Inherited from

[`BaseWalletAdapter`](BaseWalletAdapter.md).[`connected`](BaseWalletAdapter.md#connected)

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

`SignerWalletAdapter.addListener`

#### Inherited from

[`BaseWalletAdapter`](BaseWalletAdapter.md).[`addListener`](BaseWalletAdapter.md#addlistener)

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

`SignerWalletAdapter.connect`

#### Inherited from

[`BaseWalletAdapter`](BaseWalletAdapter.md).[`connect`](BaseWalletAdapter.md#connect)

***

### disconnect()

> `abstract` **disconnect**(): `Promise`\<`void`\>

#### Returns

`Promise`\<`void`\>

#### Implementation of

`SignerWalletAdapter.disconnect`

#### Inherited from

[`BaseWalletAdapter`](BaseWalletAdapter.md).[`disconnect`](BaseWalletAdapter.md#disconnect)

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

`SignerWalletAdapter.emit`

#### Inherited from

[`BaseWalletAdapter`](BaseWalletAdapter.md).[`emit`](BaseWalletAdapter.md#emit)

***

### eventNames()

> **eventNames**(): keyof [`WalletAdapterEvents`](../interfaces/WalletAdapterEvents.md)[]

Return an array listing the events for which the emitter has registered
listeners.

#### Returns

keyof [`WalletAdapterEvents`](../interfaces/WalletAdapterEvents.md)[]

#### Implementation of

`SignerWalletAdapter.eventNames`

#### Inherited from

[`BaseWalletAdapter`](BaseWalletAdapter.md).[`eventNames`](BaseWalletAdapter.md#eventnames)

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

`SignerWalletAdapter.listenerCount`

#### Inherited from

[`BaseWalletAdapter`](BaseWalletAdapter.md).[`listenerCount`](BaseWalletAdapter.md#listenercount)

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

`SignerWalletAdapter.listeners`

#### Inherited from

[`BaseWalletAdapter`](BaseWalletAdapter.md).[`listeners`](BaseWalletAdapter.md#listeners)

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

`SignerWalletAdapter.off`

#### Inherited from

[`BaseWalletAdapter`](BaseWalletAdapter.md).[`off`](BaseWalletAdapter.md#off)

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

`SignerWalletAdapter.on`

#### Inherited from

[`BaseWalletAdapter`](BaseWalletAdapter.md).[`on`](BaseWalletAdapter.md#on)

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

`SignerWalletAdapter.once`

#### Inherited from

[`BaseWalletAdapter`](BaseWalletAdapter.md).[`once`](BaseWalletAdapter.md#once)

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

`SignerWalletAdapter.removeAllListeners`

#### Inherited from

[`BaseWalletAdapter`](BaseWalletAdapter.md).[`removeAllListeners`](BaseWalletAdapter.md#removealllisteners)

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

`SignerWalletAdapter.removeListener`

#### Inherited from

[`BaseWalletAdapter`](BaseWalletAdapter.md).[`removeListener`](BaseWalletAdapter.md#removelistener)
