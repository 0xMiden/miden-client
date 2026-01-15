[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / AdviceMap

# Class: AdviceMap

Map of advice values keyed by words for script execution.

## Constructors

### Constructor

> **new AdviceMap**(): `AdviceMap`

Creates an empty advice map.

#### Returns

`AdviceMap`

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

> **insert**(`key`, `value`): [`Felt`](Felt.md)[]

Inserts a value for the given key, returning any previous value.

#### Parameters

##### key

[`Word`](Word.md)

##### value

[`FeltArray`](FeltArray.md)

#### Returns

[`Felt`](Felt.md)[]
