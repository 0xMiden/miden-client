[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / setupLogging

# Function: setupLogging()

> **setupLogging**(`logLevel`): `void`

Initializes the tracing subscriber that routes Rust log output to the
browser console. Call once per thread (main thread / Web Worker).
Subsequent calls on the same thread are harmless no-ops.

## Parameters

### logLevel

[`LogLevel`](../type-aliases/LogLevel.md)

The maximum log level to display.

## Returns

`void`
