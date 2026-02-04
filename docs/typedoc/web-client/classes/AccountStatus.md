[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / AccountStatus

# Class: AccountStatus

Represents the status of an account tracked by the client.

The status of an account may change by local or external factors.

## Methods

### \[dispose\]()

> **\[dispose\]**(): `void`

#### Returns

`void`

***

### free()

> **free**(): `void`

#### Returns

`void`

***

### isLocked()

> **isLocked**(): `boolean`

Returns `true` if the account is locked.

A locked account has a local state that doesn't match the node's state,
rendering it unusable for transactions.

#### Returns

`boolean`

***

### seed()

> **seed**(): [`Word`](Word.md)

Returns the account seed if available.

The seed is available for:
- New accounts (stored in the New status)
- Locked private accounts with nonce=0 (preserved for reconstruction)

#### Returns

[`Word`](Word.md)

***

### toString()

> **toString**(): `string`

Returns the status as a string representation.

#### Returns

`string`
