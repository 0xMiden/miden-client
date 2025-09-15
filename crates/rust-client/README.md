# Rust Client Library

Rust library, which can be used by other project to programmatically interact with the Miden rollup.

## Adding miden-client as a dependency

In order to utilize the `miden-client` library, you can add the dependency to your project's `Cargo.toml` file:

````toml
miden-client = { version = "0.11" }
````

## Crate Features

| Features     | Description                                                                                                                                               |
| ------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `tonic`      | Includes `TonicRpcClient`, a `std`-compatible Tonic client to communicate with Miden node. This relies on the `tonic` for the inner transport.  **Disabled by default.**                                                        |
| `web-tonic`  | Includes `TonicRpcClient`, a `wasm`-compatible Tonic client to communicate with the Miden node. This relies on `tonic-web-wasm-client` for the inner transport. **Disabled by default.**                                   |
| `testing`    | Enables functions meant to be used in testing environments. **Disabled by default.**             |

### Store and RpcClient implementations

The library user can provide their own implementations of `Store` and `RpcClient` traits, which can be used as components of `Client`, though it is not necessary. The `Store` trait is used to persist the state of the client, while the `RpcClient` trait is used to communicate via [gRPC](https://grpc.io/) with the Miden node.

Storage backends are provided as separate crates:
- SQLite: `miden-client-sqlite-store`, based on SQLite. For `std`-compatible environments.
- Web (WASM): `idxdb-store`, based on IndexedDB. For browser environments.

## License
This project is [MIT licensed](../../LICENSE).
