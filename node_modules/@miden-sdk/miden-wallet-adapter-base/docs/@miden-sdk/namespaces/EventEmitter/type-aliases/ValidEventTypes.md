[**@miden-sdk/miden-wallet-adapter-base**](../../../../README.md)

***

[@miden-sdk/miden-wallet-adapter-base](../../../../README.md) / [EventEmitter](../README.md) / ValidEventTypes

# Type Alias: ValidEventTypes

> **ValidEventTypes** = `string` \| `symbol` \| `object`

`object` should be in either of the following forms:
```
interface EventTypes {
  'event-with-parameters': any[]
  'event-with-example-handler': (...args: any[]) => void
}
```
