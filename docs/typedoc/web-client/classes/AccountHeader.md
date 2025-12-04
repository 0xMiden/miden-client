[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / AccountHeader

# Class: AccountHeader

A header of an account which contains information that succinctly describes the state of the
components of the account.

The [`AccountHeader`] is composed of:
- `id`: the account ID ([`AccountId`]) of the account.
- `nonce`: the nonce of the account.
- `vault_root`: a commitment to the account's vault ([`super::AssetVault`]).
- `storage_commitment`: a commitment to the account's storage ([`super::AccountStorage`]).
- `code_commitment`: a commitment to the account's code ([`super::AccountCode`]).

## Methods

### \[dispose\]()

> **\[dispose\]**(): `void`

#### Returns

`void`

***

### codeCommitment()

> **codeCommitment**(): [`Word`](Word.md)

Returns the code commitment.

#### Returns

[`Word`](Word.md)

***

### commitment()

> **commitment**(): [`Word`](Word.md)

Returns the full account commitment.

#### Returns

[`Word`](Word.md)

***

### free()

> **free**(): `void`

#### Returns

`void`

***

### id()

> **id**(): [`AccountId`](AccountId.md)

Returns the account ID.

#### Returns

[`AccountId`](AccountId.md)

***

### nonce()

> **nonce**(): [`Felt`](Felt.md)

Returns the current nonce.

#### Returns

[`Felt`](Felt.md)

***

### storageCommitment()

> **storageCommitment**(): [`Word`](Word.md)

Returns the storage commitment.

#### Returns

[`Word`](Word.md)

***

### vaultCommitment()

> **vaultCommitment**(): [`Word`](Word.md)

Returns the vault commitment.

#### Returns

[`Word`](Word.md)
