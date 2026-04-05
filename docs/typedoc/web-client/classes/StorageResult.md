[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / StorageResult

# Class: StorageResult

Result of reading a storage slot via `StorageView.getItem()`.
Works for both Value and StorageMap slots.

## Constructors

### Constructor

> **new StorageResult**(): `StorageResult`

#### Returns

`StorageResult`

## Accessors

### entries

#### Get Signature

> **get** **entries**(): `object`[]

All entries from a StorageMap slot.
Each entry has `key` (hex), `value` (hex), and `word` (parsed Word or undefined).
Returns undefined for Value slots.

##### Returns

`object`[]

***

### isMap

#### Get Signature

> **get** **isMap**(): `boolean`

True if this slot is a StorageMap.

##### Returns

`boolean`

***

### word

#### Get Signature

> **get** **word**(): [`Word`](Word.md)

The underlying Word value.

##### Returns

[`Word`](Word.md)

## Methods

### felt()

> **felt**(): [`Felt`](Felt.md)

The first Felt of the stored Word.

#### Returns

[`Felt`](Felt.md)

***

### toBigInt()

> **toBigInt**(): `bigint`

First felt as a BigInt. Preserves full u64 precision.

#### Returns

`bigint`

***

### toFelts()

> **toFelts**(): [`Felt`](Felt.md)[]

Returns all four Felts of the stored Word. Pass-through to Word.toFelts().

#### Returns

[`Felt`](Felt.md)[]

***

### toHex()

> **toHex**(): `string`

The Word's hex representation.

#### Returns

`string`

***

### toJSON()

> **toJSON**(): `string`

Returns the value as a string for JSON precision safety.

#### Returns

`string`

***

### toNumber()

> **toNumber**(): `number`

First felt as a JavaScript number.
WARNING: May lose precision for values > Number.MAX_SAFE_INTEGER.

#### Returns

`number`

***

### toString()

> **toString**(): `string`

Renders as the numeric value. Makes `{result}` work in JSX.

#### Returns

`string`

***

### valueOf()

> **valueOf**(): `number`

Allows arithmetic: `+result`, `result * 2`.

#### Returns

`number`
