[**@miden-sdk/miden-wallet-adapter-miden**](../README.md)

***

[@miden-sdk/miden-wallet-adapter-miden](../README.md) / MidenWalletAdapter

# Class: MidenWalletAdapter

## Extends

- `BaseMessageSignerWalletAdapter`

## Constructors

### Constructor

> **new MidenWalletAdapter**(`__namedParameters`): `MidenWalletAdapter`

#### Parameters

##### \_\_namedParameters

[`MidenWalletAdapterConfig`](../interfaces/MidenWalletAdapterConfig.md) = `{}`

#### Returns

`MidenWalletAdapter`

#### Overrides

`BaseMessageSignerWalletAdapter.constructor`

## Properties

### icon

> **icon**: `string` = `'data:image/png;base64,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'`

#### Overrides

`BaseMessageSignerWalletAdapter.icon`

***

### name

> **name**: `WalletName`\<`"Miden Wallet"`\> = `MidenWalletName`

#### Overrides

`BaseMessageSignerWalletAdapter.name`

***

### supportedTransactionVersions

> `readonly` **supportedTransactionVersions**: `any` = `null`

#### Overrides

`BaseMessageSignerWalletAdapter.supportedTransactionVersions`

***

### url

> **url**: `string` = `'https://miden.fi/'`

#### Overrides

`BaseMessageSignerWalletAdapter.url`

***

### prefixed

> `static` **prefixed**: `string` \| `boolean`

#### Inherited from

`BaseMessageSignerWalletAdapter.prefixed`

## Accessors

### address

#### Get Signature

> **get** **address**(): `string`

##### Returns

`string`

#### Overrides

`BaseMessageSignerWalletAdapter.address`

***

### connected

#### Get Signature

> **get** **connected**(): `boolean`

##### Returns

`boolean`

#### Inherited from

`BaseMessageSignerWalletAdapter.connected`

***

### connecting

#### Get Signature

> **get** **connecting**(): `boolean`

##### Returns

`boolean`

#### Overrides

`BaseMessageSignerWalletAdapter.connecting`

***

### privateDataPermission

#### Get Signature

> **get** **privateDataPermission**(): `string`

##### Returns

`string`

***

### publicKey

#### Get Signature

> **get** **publicKey**(): `Uint8Array`\<`ArrayBufferLike`\>

##### Returns

`Uint8Array`\<`ArrayBufferLike`\>

#### Overrides

`BaseMessageSignerWalletAdapter.publicKey`

***

### readyState

#### Get Signature

> **get** **readyState**(): `WalletReadyState`

##### Returns

`WalletReadyState`

#### Set Signature

> **set** **readyState**(`readyState`): `void`

##### Parameters

###### readyState

`WalletReadyState`

##### Returns

`void`

#### Overrides

`BaseMessageSignerWalletAdapter.readyState`

## Methods

### addListener()

> **addListener**\<`T`\>(`event`, `fn`, `context?`): `this`

#### Type Parameters

##### T

`T` *extends* keyof `WalletAdapterEvents`

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

`BaseMessageSignerWalletAdapter.addListener`

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

#### Overrides

`BaseMessageSignerWalletAdapter.connect`

***

### disconnect()

> **disconnect**(): `Promise`\<`void`\>

#### Returns

`Promise`\<`void`\>

#### Overrides

`BaseMessageSignerWalletAdapter.disconnect`

***

### emit()

> **emit**\<`T`\>(`event`, ...`args`): `boolean`

Calls each of the listeners registered for a given event.

#### Type Parameters

##### T

`T` *extends* keyof `WalletAdapterEvents`

#### Parameters

##### event

`T`

##### args

...`ArgumentMap`\<`WalletAdapterEvents`\>\[`Extract`\<`T`, keyof `WalletAdapterEvents`\>\]

#### Returns

`boolean`

#### Inherited from

`BaseMessageSignerWalletAdapter.emit`

***

### eventNames()

> **eventNames**(): keyof `WalletAdapterEvents`[]

Return an array listing the events for which the emitter has registered
listeners.

#### Returns

keyof `WalletAdapterEvents`[]

#### Inherited from

`BaseMessageSignerWalletAdapter.eventNames`

***

### importPrivateNote()

> **importPrivateNote**(`note`): `Promise`\<`string`\>

#### Parameters

##### note

`Uint8Array`

#### Returns

`Promise`\<`string`\>

#### Overrides

`BaseMessageSignerWalletAdapter.importPrivateNote`

***

### listenerCount()

> **listenerCount**(`event`): `number`

Return the number of listeners listening to a given event.

#### Parameters

##### event

keyof `WalletAdapterEvents`

#### Returns

`number`

#### Inherited from

`BaseMessageSignerWalletAdapter.listenerCount`

***

### listeners()

> **listeners**\<`T`\>(`event`): (...`args`) => `void`[]

Return the listeners registered for a given event.

#### Type Parameters

##### T

`T` *extends* keyof `WalletAdapterEvents`

#### Parameters

##### event

`T`

#### Returns

(...`args`) => `void`[]

#### Inherited from

`BaseMessageSignerWalletAdapter.listeners`

***

### off()

> **off**\<`T`\>(`event`, `fn?`, `context?`, `once?`): `this`

#### Type Parameters

##### T

`T` *extends* keyof `WalletAdapterEvents`

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

`BaseMessageSignerWalletAdapter.off`

***

### on()

> **on**\<`T`\>(`event`, `fn`, `context?`): `this`

Add a listener for a given event.

#### Type Parameters

##### T

`T` *extends* keyof `WalletAdapterEvents`

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

`BaseMessageSignerWalletAdapter.on`

***

### once()

> **once**\<`T`\>(`event`, `fn`, `context?`): `this`

Add a one-time listener for a given event.

#### Type Parameters

##### T

`T` *extends* keyof `WalletAdapterEvents`

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

`BaseMessageSignerWalletAdapter.once`

***

### removeAllListeners()

> **removeAllListeners**(`event?`): `this`

Remove all listeners, or those of the specified event.

#### Parameters

##### event?

keyof `WalletAdapterEvents`

#### Returns

`this`

#### Inherited from

`BaseMessageSignerWalletAdapter.removeAllListeners`

***

### removeListener()

> **removeListener**\<`T`\>(`event`, `fn?`, `context?`, `once?`): `this`

Remove the listeners of a given event.

#### Type Parameters

##### T

`T` *extends* keyof `WalletAdapterEvents`

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

`BaseMessageSignerWalletAdapter.removeListener`

***

### requestAssets()

> **requestAssets**(): `Promise`\<`Asset`[]\>

#### Returns

`Promise`\<`Asset`[]\>

#### Overrides

`BaseMessageSignerWalletAdapter.requestAssets`

***

### requestConsumableNotes()

> **requestConsumableNotes**(): `Promise`\<`InputNoteDetails`[]\>

#### Returns

`Promise`\<`InputNoteDetails`[]\>

#### Overrides

`BaseMessageSignerWalletAdapter.requestConsumableNotes`

***

### requestConsume()

> **requestConsume**(`transaction`): `Promise`\<`string`\>

#### Parameters

##### transaction

`MidenConsumeTransaction`

#### Returns

`Promise`\<`string`\>

#### Overrides

`BaseMessageSignerWalletAdapter.requestConsume`

***

### requestPrivateNotes()

> **requestPrivateNotes**(`noteFilterType`, `noteIds?`): `Promise`\<`InputNoteDetails`[]\>

#### Parameters

##### noteFilterType

`NoteFilterTypes`

##### noteIds?

`string`[]

#### Returns

`Promise`\<`InputNoteDetails`[]\>

#### Overrides

`BaseMessageSignerWalletAdapter.requestPrivateNotes`

***

### requestSend()

> **requestSend**(`transaction`): `Promise`\<`string`\>

#### Parameters

##### transaction

`MidenSendTransaction`

#### Returns

`Promise`\<`string`\>

#### Overrides

`BaseMessageSignerWalletAdapter.requestSend`

***

### requestTransaction()

> **requestTransaction**(`transaction`): `Promise`\<`string`\>

#### Parameters

##### transaction

`MidenTransaction`

#### Returns

`Promise`\<`string`\>

#### Overrides

`BaseMessageSignerWalletAdapter.requestTransaction`

***

### signBytes()

> **signBytes**(`message`, `kind`): `Promise`\<`Uint8Array`\<`ArrayBufferLike`\>\>

#### Parameters

##### message

`Uint8Array`

##### kind

`SignKind`

#### Returns

`Promise`\<`Uint8Array`\<`ArrayBufferLike`\>\>

#### Overrides

`BaseMessageSignerWalletAdapter.signBytes`

***

### waitForTransaction()

> **waitForTransaction**(`txId`, `timeout?`): `Promise`\<`TransactionOutput`\>

#### Parameters

##### txId

`string`

##### timeout?

`number`

#### Returns

`Promise`\<`TransactionOutput`\>

#### Overrides

`BaseMessageSignerWalletAdapter.waitForTransaction`
