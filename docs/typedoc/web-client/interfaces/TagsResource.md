[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / TagsResource

# Interface: TagsResource

## Methods

### add()

> **add**(`tag`): `Promise`\<`void`\>

Add a note tag to listen for during sync.

#### Parameters

##### tag

`number`

The numeric note tag to register.

#### Returns

`Promise`\<`void`\>

***

### list()

> **list**(): `Promise`\<`number`[]\>

List all registered note tags.

#### Returns

`Promise`\<`number`[]\>

***

### remove()

> **remove**(`tag`): `Promise`\<`void`\>

Remove a note tag so it is no longer tracked during sync.

#### Parameters

##### tag

`number`

The numeric note tag to unregister.

#### Returns

`Promise`\<`void`\>
