[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / AccountCode

# Class: AccountCode

A public interface of an account.

Account's public interface consists of a set of callable procedures, each committed to by its
root hash and paired with storage bounds (offset and size).

The full interface commitment hashes every procedure root together with its storage bounds so
that the account code uniquely captures the set of available calls.

## Methods

### \[dispose\]()

> **\[dispose\]**(): `void`

#### Returns

`void`

***

### commitment()

> **commitment**(): [`Word`](Word.md)

Returns the code commitment for the account.

#### Returns

[`Word`](Word.md)

***

### free()

> **free**(): `void`

#### Returns

`void`

***

### hasProcedure()

> **hasProcedure**(`mast_root`): `boolean`

Returns true if the account code exports a procedure with the given MAST root.

#### Parameters

##### mast\_root

[`Word`](Word.md)

#### Returns

`boolean`
