[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / SettingsResource

# Interface: SettingsResource

## Methods

### get()

> **get**\<`T`\>(`key`): `Promise`\<`T`\>

Get a setting value by key. Returns `null` if not found.

#### Type Parameters

##### T

`T` = `unknown`

#### Parameters

##### key

`string`

The setting key.

#### Returns

`Promise`\<`T`\>

***

### listKeys()

> **listKeys**(): `Promise`\<`string`[]\>

List all setting keys.

#### Returns

`Promise`\<`string`[]\>

***

### remove()

> **remove**(`key`): `Promise`\<`void`\>

Remove a setting.

#### Parameters

##### key

`string`

The setting key to remove.

#### Returns

`Promise`\<`void`\>

***

### set()

> **set**(`key`, `value`): `Promise`\<`void`\>

Set a setting value.

#### Parameters

##### key

`string`

The setting key.

##### value

`unknown`

The value to store.

#### Returns

`Promise`\<`void`\>
