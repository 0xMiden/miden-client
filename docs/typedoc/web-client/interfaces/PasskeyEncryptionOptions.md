[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / PasskeyEncryptionOptions

# Interface: PasskeyEncryptionOptions

## Properties

### credentialId?

> `optional` **credentialId**: `string`

Existing credential ID (base64url). Omit to register a new passkey.

***

### rpId?

> `optional` **rpId**: `string`

WebAuthn relying party ID. Defaults to current hostname.

***

### rpName?

> `optional` **rpName**: `string`

Relying party display name. Defaults to "Miden Client".

***

### userName?

> `optional` **userName**: `string`

User display name for the passkey. Defaults to "Miden Wallet User".
