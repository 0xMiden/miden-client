[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / ClientOptions

# Interface: ClientOptions

## Properties

### autoSync?

> `optional` **autoSync**: `boolean`

Sync state on creation (default: false).

***

### keystore?

> `optional` **keystore**: `object`

External keystore callbacks.

#### getKey

> **getKey**: [`GetKeyCallback`](../type-aliases/GetKeyCallback.md)

#### insertKey

> **insertKey**: [`InsertKeyCallback`](../type-aliases/InsertKeyCallback.md)

#### sign?

> `optional` **sign**: [`SignCallback`](../type-aliases/SignCallback.md)

Optional signing callback. When omitted, the Rust/WASM side calls `getKey`
to retrieve the secret key and signs locally. Only provide this if you need
signing to happen outside of WASM (e.g., in a remote HSM).

***

### noteTransportUrl?

> `optional` **noteTransportUrl**: `string`

Note transport URL (optional).

***

### passkeyEncryption?

> `optional` **passkeyEncryption**: `boolean` \| [`PasskeyEncryptionOptions`](PasskeyEncryptionOptions.md)

Opt-in passkey encryption for keys at rest. Pass `true` for defaults
or a `PasskeyEncryptionOptions` object to reuse an existing credential.

When `true`, checks localStorage for an existing credential and reuses it
if found; otherwise registers a new passkey (triggering a biometric prompt).

Requires Chrome 116+, Safari 18+, or Edge 116+. Firefox does NOT support PRF.
Mutually exclusive with `keystore` — if both are provided, `keystore` takes precedence.

***

### proverUrl?

> `optional` **proverUrl**: `string`

Auto-creates a remote prover from this URL.

***

### rpcUrl?

> `optional` **rpcUrl**: `string`

RPC endpoint URL. Defaults to testnet RPC.

***

### seed?

> `optional` **seed**: `string` \| `Uint8Array`

Hashed to 32 bytes via SHA-256.

***

### storeName?

> `optional` **storeName**: `string`

Store isolation key.
