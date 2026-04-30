[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / PasskeyKeystore

# Interface: PasskeyKeystore

Result of `createPasskeyKeystore()`.

## Properties

### credentialId

> **credentialId**: `string`

The credential ID (base64url) of the passkey used for this keystore.

***

### getKey

> **getKey**: [`GetKeyCallback`](../type-aliases/GetKeyCallback.md)

Decrypts and returns the secret key for a given pub key commitment.

***

### insertKey

> **insertKey**: [`InsertKeyCallback`](../type-aliases/InsertKeyCallback.md)

Encrypts and stores a secret key for a given pub key commitment.
