[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / StorageMap

# Class: StorageMap

An account storage map is a sparse merkle tree of depth [`Self::TREE_DEPTH`] (64).

It can be used to store a large amount of data in an account than would be otherwise possible
using just the account's storage slots. This works by storing the root of the map's underlying
SMT in one account storage slot. Each map entry is a leaf in the tree and its inclusion is
proven while retrieving it (e.g. via `AccountStorage::get_map_item`).

As a side-effect, this also means that _not all_ entries of the map have to be present at
transaction execution time in order to access or modify the map. It is sufficient if _just_ the
accessed/modified items are present in the advice provider.

Because the keys of the map are user-chosen and thus not necessarily uniformly distributed, the
tree could be imbalanced and made less efficient. To mitigate that, the keys used in the storage
map are hashed before they are inserted into the SMT, which creates a uniform distribution. The
original keys are retained in a separate map. This causes redundancy but allows for
introspection of the map, e.g. by querying the set of stored (original) keys which is useful in
debugging and explorer scenarios.

## Constructors

### Constructor

> **new StorageMap**(): `StorageMap`

Creates an empty storage map.

#### Returns

`StorageMap`

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

### insert()

> **insert**(`key`, `value`): [`Word`](Word.md)

Inserts a key/value pair, returning any previous value.

#### Parameters

##### key

[`Word`](Word.md)

##### value

[`Word`](Word.md)

#### Returns

[`Word`](Word.md)
