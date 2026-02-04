[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / AccountReader

# Class: AccountReader

Provides lazy access to account data.

`AccountReader` executes queries lazily - each method call fetches fresh data
from storage, ensuring you always see the current state.

## Methods

### \[dispose\]()

> **\[dispose\]**(): `void`

#### Returns

`void`

***

### accountId()

> **accountId**(): [`AccountId`](AccountId.md)

Returns the account ID.

#### Returns

[`AccountId`](AccountId.md)

***

### addresses()

> **addresses**(): `Promise`\<[`Address`](Address.md)[]\>

Retrieves the addresses associated with this account.

#### Returns

`Promise`\<[`Address`](Address.md)[]\>

***

### codeCommitment()

> **codeCommitment**(): `Promise`\<[`Word`](Word.md)\>

Retrieves the code commitment (hash of the account code).

#### Returns

`Promise`\<[`Word`](Word.md)\>

***

### commitment()

> **commitment**(): `Promise`\<[`Word`](Word.md)\>

Retrieves the account commitment (hash of the full state).

#### Returns

`Promise`\<[`Word`](Word.md)\>

***

### free()

> **free**(): `void`

#### Returns

`void`

***

### getBalance()

> **getBalance**(`faucet_id`): `Promise`\<`bigint`\>

Retrieves the balance of a fungible asset in the account's vault.

Returns 0 if the asset is not present in the vault.

#### Parameters

##### faucet\_id

[`AccountId`](AccountId.md)

#### Returns

`Promise`\<`bigint`\>

***

### getStorageItem()

> **getStorageItem**(`slot_name`): `Promise`\<[`Word`](Word.md)\>

Retrieves a storage slot value by name.

For `Value` slots, returns the stored word.
For `Map` slots, returns the map root.

#### Parameters

##### slot\_name

`string`

#### Returns

`Promise`\<[`Word`](Word.md)\>

***

### getStorageMapItem()

> **getStorageMapItem**(`slot_name`, `key`): `Promise`\<[`Word`](Word.md)\>

Retrieves a value from a storage map slot by name and key.

#### Parameters

##### slot\_name

`string`

##### key

[`Word`](Word.md)

#### Returns

`Promise`\<[`Word`](Word.md)\>

***

### header()

> **header**(): `Promise`\<[`AccountHeader`](AccountHeader.md)\>

Retrieves the account header.

#### Returns

`Promise`\<[`AccountHeader`](AccountHeader.md)\>

***

### isLocked()

> **isLocked**(): `Promise`\<`boolean`\>

Returns whether the account is locked.

#### Returns

`Promise`\<`boolean`\>

***

### isNew()

> **isNew**(): `Promise`\<`boolean`\>

Returns whether the account is new.

#### Returns

`Promise`\<`boolean`\>

***

### nonce()

> **nonce**(): `Promise`\<[`Felt`](Felt.md)\>

Retrieves the current account nonce.

#### Returns

`Promise`\<[`Felt`](Felt.md)\>

***

### seed()

> **seed**(): `Promise`\<[`Word`](Word.md)\>

Retrieves the account seed (if available for new/locked accounts).

#### Returns

`Promise`\<[`Word`](Word.md)\>

***

### storageCommitment()

> **storageCommitment**(): `Promise`\<[`Word`](Word.md)\>

Retrieves the storage commitment (root of the storage tree).

#### Returns

`Promise`\<[`Word`](Word.md)\>

***

### vaultRoot()

> **vaultRoot**(): `Promise`\<[`Word`](Word.md)\>

Retrieves the vault root (root of the asset vault tree).

#### Returns

`Promise`\<[`Word`](Word.md)\>
