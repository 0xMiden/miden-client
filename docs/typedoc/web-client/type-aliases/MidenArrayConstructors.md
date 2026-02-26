[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / MidenArrayConstructors

# Type Alias: MidenArrayConstructors

> **MidenArrayConstructors** = `` { [K in keyof typeof WasmExports as K extends `${string}Array` ? K : never]: typeof WasmExports[K] } ``
