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

#### sign

> **sign**: [`SignCallback`](../type-aliases/SignCallback.md)

***

### noteTransportUrl?

> `optional` **noteTransportUrl**: `"testnet"` \| `"devnet"` \| `string` & `object`

Note transport endpoint. Accepts shorthands or a raw URL:
- `"testnet"` — Miden testnet transport (`https://transport.miden.io`)
- `"devnet"` — Miden devnet transport (`https://transport.devnet.miden.io`)
- any other string — treated as a raw note transport endpoint URL

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
