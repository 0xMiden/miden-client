---
title: Peer-to-peer transfer
sidebar_position: 4
---

In this section, we show you how to execute transactions and send funds to another account using the Miden CLI and public notes.

:::info Prerequisite
Complete the [Create account](./create-account-use-faucet.md) tutorial first. You should have a funded account and should *not* have reset the state of your local client.
:::

## Create a second client

This is an alternative to the [private P2P transactions](./p2p-private.md) process.

In this tutorial, we use two different clients to simulate two different remote users who don't share local state. Use two terminals, each with their own state (using their own `miden-client.toml`).

1. Create a new directory to store the new client.

    ```sh
    mkdir miden-client-2
    cd miden-client-2
    ```

2. On the new client, create a new account with public storage:

    ```sh
    miden-client new-wallet --mutable -s public
    ```

    We refer to this account as _Account C_. The account's storage mode is `public`, meaning its details are public and its latest state can be retrieved from the node.

3. List and view the account:

    ```sh
    miden-client account -l
    ```

## Transfer assets between accounts

1. Switch back to the first `miden-client` directory. From the first client, run:

    ```sh
    miden-client send --sender <basic-account-id-A> --target <basic-account-id-C> --asset 50::<faucet-account-id> --note-type public
    ```

    :::note
    The faucet account ID can be found on the [Miden faucet website](https://testnet.miden.io/) under the title **Miden faucet**.
    :::

    This generates a Pay-to-ID (`P2ID`) note containing `50` tokens. As the note is public, the second account can receive the details by syncing with the node.

2. Sync the second client:

    ```sh
    miden-client sync
    ```

3. View the received note:

    ```sh
    miden-client notes --list
    ```

4. Have account C consume the note:

    ```sh
    miden-client consume-notes --account <regular-account-ID-C> <input-note-id>
    ```

    :::tip
    It's possible to use a short version of the note ID: 7 characters after the `0x` is sufficient, e.g. `0x6ae613a`.
    :::

Account C has now consumed the note. Verify the new assets:

```sh
miden-client account --show <account-ID>
```

## Clear state

All state is maintained in `store.sqlite3`, located in the directory defined in `miden-client.toml`. To clear all state, delete this file. It recreates on any command execution.
