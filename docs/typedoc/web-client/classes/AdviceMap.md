[**@demox-labs/miden-sdk**](../README.md)

***

[@demox-labs/miden-sdk](../README.md) / AdviceMap

# Class: AdviceMap

Map of advice values made available to the VM during script execution.

## Constructors

### Constructor

> **new AdviceMap**(): `AdviceMap`

Creates an empty advice map.

#### Returns

`AdviceMap`

## Methods

### free()

> **free**(): `void`

#### Returns

`void`

***

### insert()

> **insert**(`key`, `value`): [`Felt`](Felt.md)[]

Inserts a list of field elements for the given key and returns any previous mapping.

#### Parameters

##### key

[`Word`](Word.md)

##### value

[`FeltArray`](FeltArray.md)

#### Returns

[`Felt`](Felt.md)[]
