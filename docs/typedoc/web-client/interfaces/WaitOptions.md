[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / WaitOptions

# Interface: WaitOptions

## Properties

### interval?

> `optional` **interval**: `number`

Polling interval in ms (default: 5_000).

***

### onProgress()?

> `optional` **onProgress**: (`status`) => `void`

#### Parameters

##### status

[`WaitStatus`](../type-aliases/WaitStatus.md)

#### Returns

`void`

***

### timeout?

> `optional` **timeout**: `number`

Timeout in ms (default: 60_000). Set to 0 to disable timeout and poll indefinitely.
