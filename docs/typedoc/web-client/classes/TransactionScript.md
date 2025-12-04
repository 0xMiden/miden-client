[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / TransactionScript

# Class: TransactionScript

A transaction script is a program that is executed in a transaction after all input notes have
been executed.

The [`TransactionScript`] object is composed of:
- An executable program defined by a MAST forest and an associated entrypoint.
- A set of transaction script inputs defined by a map of key-value inputs that are loaded into
  the advice inputs' map such that the transaction script can access them.

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

### root()

> **root**(): [`Word`](Word.md)

Returns the MAST root commitment of the transaction script.

#### Returns

[`Word`](Word.md)
