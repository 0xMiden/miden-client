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

#### Returns

`Promise`\<`void`\>

***

### set()

> **set**(`key`, `value`): `Promise`\<`void`\>

Set a setting value.

#### Parameters

##### key

`string`

##### value

`unknown`

#### Returns

`Promise`\<`void`\>
