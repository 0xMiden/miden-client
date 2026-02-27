[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / createPasskeyKeystore

# Function: createPasskeyKeystore()

> **createPasskeyKeystore**(`storeName`, `options?`): `Promise`\<[`PasskeyKeystore`](../interfaces/PasskeyKeystore.md)\>

Creates a passkey-encrypted keystore backed by WebAuthn PRF.

Registers a new passkey or authenticates with an existing one (biometric prompt),
then returns `getKey`/`insertKey` callbacks that transparently encrypt/decrypt
secret keys using the PRF-derived wrapping key.

## Parameters

### storeName

`string`

### options?

[`PasskeyEncryptionOptions`](../interfaces/PasskeyEncryptionOptions.md)

## Returns

`Promise`\<[`PasskeyKeystore`](../interfaces/PasskeyKeystore.md)\>
