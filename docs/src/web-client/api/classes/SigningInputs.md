---
title: SigningInputs
draft: true
---

[**@demox-labs/miden-sdk**](../index)

***

[@demox-labs/miden-sdk](../index) / SigningInputs

# Class: SigningInputs

## Properties

### variantType

> `readonly` **variantType**: `string`

## Methods

### free()

> **free**(): `void`

#### Returns

`void`

***

### toCommitment()

> **toCommitment**(): [`Word`](Word)

#### Returns

[`Word`](Word)

***

### toElements()

> **toElements**(): [`Felt`](Felt)[]

#### Returns

[`Felt`](Felt)[]

***

### newArbitrary()

> `static` **newArbitrary**(`felts`): `SigningInputs`

#### Parameters

##### felts

[`Felt`](Felt)[]

#### Returns

`SigningInputs`

***

### newBlind()

> `static` **newBlind**(`word`): `SigningInputs`

#### Parameters

##### word

[`Word`](Word)

#### Returns

`SigningInputs`

***

### newTransactionSummary()

> `static` **newTransactionSummary**(`summary`): `SigningInputs`

#### Parameters

##### summary

[`TransactionSummary`](TransactionSummary)

#### Returns

`SigningInputs`
