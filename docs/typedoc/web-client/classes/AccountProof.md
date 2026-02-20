[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / AccountProof

# Class: AccountProof

Proof of existence of an account's state at a specific block number, as returned by the node.

For public accounts, this includes the account header, storage slot values and account code.
For private accounts, only the account commitment and merkle proof are available.

## Methods

### \[dispose\]()

> **\[dispose\]**(): `void`

#### Returns

`void`

***

### accountCode()

> **accountCode**(): [`AccountCode`](AccountCode.md)

Returns the account code, if available (public accounts only).

#### Returns

[`AccountCode`](AccountCode.md)

***

### accountCommitment()

> **accountCommitment**(): [`Word`](Word.md)

Returns the account commitment (hash of the full state).

#### Returns

[`Word`](Word.md)

***

### accountHeader()

> **accountHeader**(): [`AccountHeader`](AccountHeader.md)

Returns the account header, if available (public accounts only).

#### Returns

[`AccountHeader`](AccountHeader.md)

***

### accountId()

> **accountId**(): [`AccountId`](AccountId.md)

Returns the account ID.

#### Returns

[`AccountId`](AccountId.md)

***

### blockNum()

> **blockNum**(): `number`

Returns the block number at which this proof was retrieved.

#### Returns

`number`

***

### free()

> **free**(): `void`

#### Returns

`void`

***

### getStorageSlotValue()

> **getStorageSlotValue**(`slot_name`): [`Word`](Word.md)

Returns the value of a storage slot by name, if available.

For `Value` slots, this returns the stored word.
For `Map` slots, this returns the map root commitment.

Returns `undefined` if the account is private or the slot name is not found.

#### Parameters

##### slot\_name

`string`

#### Returns

[`Word`](Word.md)

***

### numStorageSlots()

> **numStorageSlots**(): `number`

Returns the number of storage slots, if available (public accounts only).

#### Returns

`number`
