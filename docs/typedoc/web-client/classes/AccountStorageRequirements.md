[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / AccountStorageRequirements

# Class: AccountStorageRequirements

Wrapper around storage requirements returned by RPC endpoints.

## Constructors

### Constructor

> **new AccountStorageRequirements**(): `AccountStorageRequirements`

Creates empty storage requirements.

#### Returns

`AccountStorageRequirements`

## Methods

### free()

> **free**(): `void`

#### Returns

`void`

***

### fromSlotAndKeysArray()

> `static` **fromSlotAndKeysArray**(`slots_and_keys`): `AccountStorageRequirements`

Builds requirements from a list of slot/key pairs.

#### Parameters

##### slots\_and\_keys

[`SlotAndKeys`](SlotAndKeys.md)[]

#### Returns

`AccountStorageRequirements`
