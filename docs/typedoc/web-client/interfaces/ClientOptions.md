[**@miden-sdk/miden-sdk**](../README.md)

***

[@miden-sdk/miden-sdk](../README.md) / ClientOptions

# Interface: ClientOptions

## Properties

### autoSync?

> `optional` **autoSync**: `boolean`

Sync state on creation (default: false).

***

### debugMode?

> `optional` **debugMode**: `boolean`

Enable debug mode for transaction execution (default: false).

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

> `optional` **noteTransportUrl**: `"testnet"` \| `"devnet"` \| `string` & `object`

Note transport endpoint. Accepts shorthands or a raw URL:
- `"testnet"` — Miden testnet transport (`https://transport.miden.io`)
- `"devnet"` — Miden devnet transport (`https://transport.devnet.miden.io`)
- any other string — treated as a raw note transport endpoint URL

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

> `optional` **proverUrl**: `"testnet"` \| `"devnet"` \| `"local"` \| `string` & `object`

Prover to use for transactions. Accepts shorthands or a raw URL:
- `"local"` — local (in-browser) prover
- `"devnet"` — Miden devnet remote prover
- `"testnet"` — Miden testnet remote prover
- any other string — treated as a raw remote prover URL

***

### rpcUrl?

> `optional` **rpcUrl**: `"testnet"` \| `"devnet"` \| `"localhost"` \| `"local"` \| `string` & `object`

RPC endpoint. Accepts shorthands or a raw URL:
- `"testnet"` — Miden testnet RPC (`https://rpc.testnet.miden.io`)
- `"devnet"` — Miden devnet RPC (`https://rpc.devnet.miden.io`)
- `"localhost"` / `"local"` — local node (`http://localhost:57291`)
- any other string — treated as a raw RPC endpoint URL
Defaults to the SDK testnet RPC if omitted.

***

### seed?

> `optional` **seed**: `string` \| `Uint8Array`

Hashed to 32 bytes via SHA-256.

***

### storeName?

> `optional` **storeName**: `string`

Store isolation key.
