---
title: Create account
sidebar_position: 3
---

In this section, we show you how to create a new local Miden account and how to receive funds from the public Miden faucet website.

## Configure the Miden CLI

The Miden CLI facilitates interaction with the Miden rollup and provides a way to execute and prove transactions.

1. If you haven't already done so, open your terminal and create a new directory to store the Miden Client state.

   ```sh
   mkdir miden-client
   cd miden-client
   ```

2. Install the Miden CLI.

   ```sh
   cargo install miden-client-cli --locked
   ```

3. Verify the installation:

   ```sh
   miden-client --version
   ```

## Create a new Miden account

1. Create a new account of type `mutable` using the following command:

   ```sh
   miden-client new-wallet --mutable
   ```

2. List all created accounts by running the following command:

   ```sh
   miden-client account -l
   ```

Save the account ID for a future step.

## Request tokens from the public faucet

1. Navigate to the [Miden faucet website](https://faucet.testnet.miden.io/).

2. Copy the **Account ID** printed by the `miden-client account -l` command in the previous step.

3. Paste this ID into the **Request test tokens** input field on the faucet website and click **Send Private Note**.

:::tip
You can also click **Send Public Note**. If you do this, the note's details will be public and you will not need to download and import it, so you can skip to [Sync the client](#sync-the-client).
:::

4. After a few seconds your browser should download a file called `note.mno` (mno = Miden note). It contains the funds the faucet sent to your address.

5. Save this file on your computer, you will need it for the next step.

## Import the note into the Miden Client

1. Import the private note that you have received:

   ```sh
   miden-client import <path-to-note>/note.mno
   ```

2. View the note's information:

   ```sh
   miden-client notes
   ```

:::tip The importance of syncing
The note is listed as `Expected` because you have received a private note but have not yet synced your view of the rollup to check that the note is the result of a valid transaction. Before consuming the note, you need to update your view of the rollup by syncing.
:::

### Sync the client

Run the `sync` command periodically to keep informed about updates on the node:

```sh
miden-client sync
```

You will see something like this as output:

```sh
State synced to block 179672
New public notes: 0
Committed notes: 1
Tracked notes consumed: 0
Tracked accounts updated: 0
Locked accounts: 0
Committed transactions: 0
```

## Consume the note and receive the funds

1. After syncing, the note should have a `Committed` status, confirming it exists at the rollup level:

   ```sh
   miden-client notes
   ```

2. Find your account and note ID:

   ```sh
   miden-client account
   miden-client notes
   ```

3. Consume the note and add the funds to your account:

   ```sh
   miden-client consume-notes --account <Account-Id> <Note-Id>
   ```

4. The note is now `Processing`. Sync again to confirm:

   ```sh
   miden-client sync
   ```

5. After syncing, the note should show as `Consumed`:

   ```sh
   miden-client notes
   ```

You just created a client-side zero-knowledge proof locally on your machine and submitted it to the Miden rollup.

:::tip
You only need to copy the first 7 characters after `0x` of the Note ID.
:::

## View confirmations

View your updated account's vault:

```sh
miden-client account --show <Account-Id>
```

You should now see your account's vault containing the funds sent by the faucet.

## Debugging tips

- Need a fresh start? All state is maintained in `store.sqlite3`, located in the directory defined in `miden-client.toml`. Delete this file to clear all state — it recreates on any command execution.
- Only execute `miden-client` from the directory containing your `miden-client.toml`.
