[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / TransactionScriptInputPair

# Class: TransactionScriptInputPair

A script argument represented as a word plus additional felts.

## Constructors

### Constructor

> **new TransactionScriptInputPair**(`word`, `felts`): `TransactionScriptInputPair`

Creates a new script input pair.

#### Parameters

##### word

[`Word`](Word.md)

##### felts

[`FeltArray`](FeltArray.md)

#### Returns

`TransactionScriptInputPair`

## Methods

### \[dispose\]()

> **\[dispose\]**(): `void`

#### Returns

`void`

***

### felts()

> **felts**(): [`FeltArray`](FeltArray.md)

Returns the remaining felts for the input.

#### Returns

[`FeltArray`](FeltArray.md)

***

### free()

> **free**(): `void`

#### Returns

`void`

***

### word()

> **word**(): [`Word`](Word.md)

Returns the word part of the input.

#### Returns

[`Word`](Word.md)
