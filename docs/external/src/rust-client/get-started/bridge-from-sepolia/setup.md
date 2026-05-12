---
title: Setup
sidebar_position: 1
---

This section prepares the tools, accounts, and helper configuration used to bridge ETH
between Ethereum Sepolia and the Miden testnet (Bali).

By the end of setup you will have:

- A Miden wallet that can receive Sepolia deposits and submit Miden withdrawals.
- A funded Sepolia account stored in a Foundry keystore.
- Local bridge helper scripts configured for your accounts.

This tutorial assumes you have already completed
[Create account](../create-account-use-faucet.md) or otherwise have a working
`miden-client` install. If not, set the client up first.

## Endpoints and addresses

| Item | Value |
|---|---|
| Sepolia bridge contract | `0x1348947e282138d8f377b467f7d9c2eb0f335d1f` |
| Miden testnet RPC | `https://rpc.testnet.miden.io:443` |
| Bridge service REST API | `https://miden-testnet-bridge.dev.eu-north-3.gateway.fm/api` |
| Destination network ID (Miden on Bali) | `73` |
| L2 chain ID | `1259691107` |
| Miden bridge account (testnet v0.14) | `mtst1aqn4y5pyessyw5rukd0wnmgq6srmn7np` |
| Miden ETH faucet account (testnet v0.14) | `mtst1az5g5k0tj7vsqcrp90z2dshsmskhyely` |
| Sepolia public RPC (fallback) | `https://ethereum-sepolia-rpc.publicnode.com` |

## Prerequisites

- A funded Sepolia EOA stored in a password-protected Foundry keystore. This account
  broadcasts the L1 deposit and pays gas on `claimAsset` in the reverse direction.
- [Foundry](https://book.getfoundry.sh/getting-started/installation), specifically
  `cast`, on your `$PATH`.
- `curl` and `python3` for the deposit status helper.
- A working `miden-client` install (see [CLI setup](../../cli/index.md)).
- For the Miden to Sepolia direction only: use the Bali Miden bridge account and Miden
  ETH faucet account listed above.

## Initialise the client and create a destination wallet

If you have not already, initialise the client against the testnet and create a wallet
to receive the bridged funds:

```sh
miden-client init --network testnet
miden-client sync
miden-client new-wallet --deploy
```

The `new-wallet` output prints the new account's ID, for example:

```sh
Successfully created new wallet.
To view account details execute miden-client account -s 0xc0ffee...c0ffee
```

Copy the 30-hex-character account ID. You will pass it as the bridge destination for
Sepolia to Miden deposits and as the withdrawing wallet for Miden to Sepolia transfers.

This step also creates the local Miden client data directory used by the reverse bridge
flow. With the default global configuration, that directory is `~/.miden`.

## Create and fund a Sepolia keystore wallet

Create a Foundry keystore:

```sh
KEYSTORE_DIR=./
ACCOUNT_NAME=miden-bali-sepolia
cast wallet new "$KEYSTORE_DIR" "$ACCOUNT_NAME"
```

Use a Sepolia faucet, such as
[Google Cloud's Sepolia faucet](https://cloud.google.com/application/web3/faucet/ethereum/sepolia),
to send test ETH to the new address so you can make a deposit to the L1 bridge
contract.

## Download the helpers

The helper scripts live in the `scripts/` directory next to these docs. Download them
into a local working directory and make them executable:

```sh
mkdir -p bali-bridge
cd bali-bridge
curl -O https://raw.githubusercontent.com/0xMiden/miden-client/main/docs/external/src/rust-client/get-started/bridge-from-sepolia/scripts/bali-l1-deposit.sh
curl -O https://raw.githubusercontent.com/0xMiden/miden-client/main/docs/external/src/rust-client/get-started/bridge-from-sepolia/scripts/bali-l1-deposit-status.sh
curl -O https://raw.githubusercontent.com/0xMiden/miden-client/main/docs/external/src/rust-client/get-started/bridge-from-sepolia/scripts/bali-l2-withdraw.sh
curl -O https://raw.githubusercontent.com/0xMiden/miden-client/main/docs/external/src/rust-client/get-started/bridge-from-sepolia/scripts/bali-bridge.conf.example
chmod +x bali-l1-deposit.sh bali-l1-deposit-status.sh bali-l2-withdraw.sh
cp bali-bridge.conf.example bali-bridge.conf
```

All helpers read shared settings from `bali-bridge.conf`.

Edit `bali-bridge.conf` and fill in the Miden account ID you created earlier:

```sh
MIDEN_ACCOUNT_ID=<account-id-from-miden-client-new-wallet>
```

The config file also contains other bridge constants, so edit the file for every value
you want to change.

By default the helpers run in `DRY_RUN=1` mode: they print the exact command they would
issue without broadcasting. Use this to sanity-check inputs before spending gas or
submitting a withdrawal. `DRY_RUN` is intentionally not part of `bali-bridge.conf`; pass
it only for the command you are running.
