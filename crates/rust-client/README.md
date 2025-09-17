# Rust Client Library

Rust library, which can be used by other project to programmatically interact with the Miden rollup.

## Adding miden-client as a dependency

In order to utilize the `miden-client` library, you can add the dependency to your project's `Cargo.toml` file:

````toml
miden-client = { version = "0.11" }
````

## Crate Features

| Features  | Description |
| --------- | ----------- |
| `std`     | Enables std support and concurrent execution in `miden-tx`. Enabled by default for native targets. |
| `testing` | Enables functions meant to be used in testing environments. Disabled by default. |

RPC transport is selected automatically by target:
- Native targets use `tonic` transport with TLS.
- `wasm32` targets use `tonic-web-wasm-client`.

### Store and RpcClient implementations

The library user can provide their own implementations of `Store` and `RpcClient` traits, which can be used as components of `Client`, though it is not necessary. The `Store` trait is used to persist the state of the client, while the `RpcClient` trait is used to communicate via [gRPC](https://grpc.io/) with the Miden node.

Storage backends are provided as separate crates:
- SQLite: `miden-client-sqlite-store`, based on SQLite. For `std`-compatible environments.
- Web (WASM): `idxdb-store`, based on IndexedDB. For browser environments.

## License
This project is [MIT licensed](../../LICENSE).
