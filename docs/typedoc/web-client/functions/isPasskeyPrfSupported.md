[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / isPasskeyPrfSupported

# Function: isPasskeyPrfSupported()

> **isPasskeyPrfSupported**(): `Promise`\<`boolean`\>

Returns `true` if the current browser supports WebAuthn with the PRF extension,
which is required for passkey-based key encryption.

Supported browsers: Chrome 116+, Safari 18+, Edge 116+. Firefox does NOT support PRF.

## Returns

`Promise`\<`boolean`\>
