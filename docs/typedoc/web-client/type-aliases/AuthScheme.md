[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / AuthScheme

# Type Alias: AuthScheme

> **AuthScheme** = *typeof* [`AuthScheme`](../variables/AuthScheme.md)\[keyof *typeof* [`AuthScheme`](../variables/AuthScheme.md)\]

Union of all string values in the AuthScheme const. Merges with the
`AuthScheme` value so `authScheme?: AuthScheme` resolves to
`"falcon" | "ecdsa"` in type position while `AuthScheme.Falcon` /
`AuthScheme.ECDSA` still work in value position.
