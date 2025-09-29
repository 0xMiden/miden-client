[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / SigningInputs

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

> **toCommitment**(): [`Word`](Word.md)

#### Returns

[`Word`](Word.md)

***

### toElements()

> **toElements**(): [`Felt`](Felt.md)[]

#### Returns

[`Felt`](Felt.md)[]

***

### newArbitrary()

> `static` **newArbitrary**(`felts`): `SigningInputs`

#### Parameters

##### felts

[`Felt`](Felt.md)[]

#### Returns

`SigningInputs`

***

### newBlind()

> `static` **newBlind**(`word`): `SigningInputs`

#### Parameters

##### word

[`Word`](Word.md)

#### Returns

`SigningInputs`

***

### newTransactionSummary()

> `static` **newTransactionSummary**(`summary`): `SigningInputs`

#### Parameters

##### summary

[`TransactionSummary`](TransactionSummary.md)

#### Returns

`SigningInputs`
