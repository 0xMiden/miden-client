[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / resolveAuthScheme

# Function: resolveAuthScheme()

> **resolveAuthScheme**(`scheme?`): `number`

Resolves an `AuthScheme` string to the numeric value expected by low-level
wasm-bindgen methods such as
`AccountComponent.createAuthComponentFromCommitment(commitment, scheme)`.

## Parameters

### scheme?

[`AuthScheme`](../type-aliases/AuthScheme.md)

`AuthScheme.Falcon` or `AuthScheme.ECDSA`. Defaults to `"falcon"`.

## Returns

`number`

The numeric AuthScheme enum value.
