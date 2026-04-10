---
title: Private peer-to-peer transfer
sidebar_position: 5
---

In this section, we show you how to make private transactions and send funds to another account using the Miden CLI.

:::info Prerequisite
Complete the [Create account](./create-account-use-faucet.md) tutorial first. You should have a funded account and should *not* have reset the state of your local client.
:::

## Create a second account

1. Create a second account to send funds to. Previously, we created a `mutable` account (`Account A`). Now, create another `mutable` account (`Account B`):

   ```sh
   miden-client new-wallet --mutable
   ```

2. List the newly created accounts:

   ```sh
   miden-client account -l
   ```

   You should see two accounts.

## Transfer assets between accounts

1. Transfer tokens from Account A to Account B:

   ```sh
   miden-client send --sender <regular-account-id-A> --target <regular-account-id-B> --asset 50::<faucet-account-id> --note-type private
   ```

   :::note
   The faucet account ID can be found on the [Miden faucet website](https://testnet.miden.io/) under the title **Miden faucet**.
   :::

   This generates a private Pay-to-ID (`P2ID`) note containing `50` tokens.

2. Sync the accounts:

   ```sh
   miden-client sync
   ```

3. Get the note ID:

   ```sh
   miden-client notes
   ```

4. Have Account B consume the note:

   ```sh
   miden-client consume-notes --account <regular-account-ID-B> <input-note-id>
   ```

   :::tip
   It's possible to use a short version of the note ID: 7 characters after the `0x` is sufficient, e.g. `0x6ae613a`.
   :::

5. Verify both accounts:

   ```sh
   miden-client account --show <regular-account-ID-B>
   miden-client account --show <regular-account-ID-A>
   ```

## Using the note transport network

The steps above assume that the client owns both accounts. To exchange notes with other users, the note transport network can be used. The sender (`Account A`) will need the address (bech32 string) of the recipient (`Account B`).

After creating the note (step 1 above), get the created note ID with `miden-client notes --list`. Then send that note through the note transport network:

```sh
miden-client notes --send <note-id> <address-B>
```

The recipient can then fetch the note:

```sh
miden-client notes --fetch
```

The note will then be available to be consumed.

:::note
The client will fetch notes for tracked note tags. By default, note tags are derived from the recipient's account ID. For increased privacy, use random tags and track them with `miden-client tags --add <tag>`.
:::

## Clear data

All state is maintained in `store.sqlite3`, located in the directory defined in `miden-client.toml`. To clear all state, delete this file. It recreates on any command execution.
