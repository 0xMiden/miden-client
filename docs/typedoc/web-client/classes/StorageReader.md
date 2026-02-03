[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / StorageReader

# Class: StorageReader

Provides convenient, name-based access to account storage slots.

`StorageReader` executes queries lazily - each method call fetches only the
requested slot from storage.

# Example (JavaScript)
```javascript
// Get a storage reader for an account
const reader = client.newStorageReader(accountId);

// Read a value slot
const metadata = await reader.getItem("token_metadata");

// Read from a map slot
const balance = await reader.getMapItem("balances", userKey);
```

## Methods

### \[dispose\]()

> **\[dispose\]**(): `void`

#### Returns

`void`

***

### accountId()

> **accountId**(): [`AccountId`](AccountId.md)

Returns the account ID this reader is associated with.

#### Returns

[`AccountId`](AccountId.md)

***

### free()

> **free**(): `void`

#### Returns

`void`

***

### getItem()

> **getItem**(`slot_name`): `Promise`\<[`Word`](Word.md)\>

Retrieves a storage slot value by name.

For `Value` slots, returns the stored word.
For `Map` slots, returns the map root.

# Arguments
* `slot_name` - The name of the storage slot.

# Errors
Returns an error if the slot is not found.

#### Parameters

##### slot\_name

`string`

#### Returns

`Promise`\<[`Word`](Word.md)\>

***

### getMapItem()

> **getMapItem**(`slot_name`, `key`): `Promise`\<[`Word`](Word.md)\>

Retrieves a value from a storage map slot by name and key.

# Arguments
* `slot_name` - The name of the storage map slot.
* `key` - The key within the map.

# Errors
Returns an error if the slot is not found or is not a map.

#### Parameters

##### slot\_name

`string`

##### key

[`Word`](Word.md)

#### Returns

`Promise`\<[`Word`](Word.md)\>
