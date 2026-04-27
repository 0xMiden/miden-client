[**@miden-sdk/miden-wallet-adapter-base**](../../../../README.md)

***

[@miden-sdk/miden-wallet-adapter-base](../../../../README.md) / [EventEmitter](../README.md) / ArgumentMap

# Type Alias: ArgumentMap\<T\>

> **ArgumentMap**\<`T`\> = `{ [K in keyof T]: T[K] extends (args: any[]) => void ? Parameters<T[K]> : T[K] extends any[] ? T[K] : any[] }`

## Type Parameters

### T

`T` *extends* `object`
