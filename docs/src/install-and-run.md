## Software prerequisites

- [Rust installation](https://www.rust-lang.org/tools/install) minimum version 1.87.

## Install the client

We currently recommend installing and running the client with the [`concurrent`](#concurrent-feature) feature.

Run the following command to install the miden-client:

```sh
cargo install miden-cli --features concurrent
```

This installs the `miden` binary (at `~/.cargo/bin/miden`) with the [`concurrent`](#concurrent-feature) feature.

### `Concurrent` feature

The `concurrent` flag enables optimizations that result in faster transaction execution and proving times.

## Run the client

If the install worked correctly, you should be able to check the version by running:

```sh
miden --version
```

Once installed, you may run:
```sh
miden --help
```

This will show you the available commands and options for the client.

An more in depth tutorial can be fund in the [Getting started section](./get-started/prerequisites.md).
