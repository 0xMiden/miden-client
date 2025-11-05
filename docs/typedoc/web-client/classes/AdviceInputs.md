[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / AdviceInputs

# Class: AdviceInputs

Advice inputs passed into the Miden VM at transaction execution time.

## Methods

### free()

> **free**(): `void`

#### Returns

`void`

***

### mappedValues()

> **mappedValues**(`key`): [`Felt`](Felt.md)[]

Returns the list stored under the provided key in the advice map.

#### Parameters

##### key

[`Word`](Word.md)

#### Returns

[`Felt`](Felt.md)[]

***

### stack()

> **stack**(): [`Felt`](Felt.md)[]

Returns the advice stack contents as field elements.

#### Returns

[`Felt`](Felt.md)[]
