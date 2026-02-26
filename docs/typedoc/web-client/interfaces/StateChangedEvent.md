[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / StateChangedEvent

# Interface: StateChangedEvent

Payload delivered to [WebClient.onStateChanged](../classes/WebClient.md#onstatechanged) listeners.

## Properties

### operation?

> `optional` **operation**: `string`

The mutating operation that triggered the event.

***

### storeName

> **storeName**: `string`

The store / database name that was mutated.

***

### type

> **type**: `"stateChanged"`
