[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / AccountCode

# Class: AccountCode

Code commitment and metadata for an account.

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
