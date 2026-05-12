---
title: Bridge ETH from Sepolia
sidebar_position: 4
---

In this section, we show you how to bridge ETH between Ethereum Sepolia
and the Miden testnet (Bali) using the `miden-client` CLI and a small
Foundry-based helper script. By the end you will have moved real
Sepolia ETH into a Miden account and (optionally) bridged it back.

This tutorial assumes you have already completed
[Create account](./create-account-use-faucet.md) or otherwise have a
working `miden-client` install. If not, set the client up first.

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

- A funded Sepolia EOA stored in a password-protected Foundry keystore.
  This account broadcasts the L1 deposit and pays gas on `claimAsset`
  in the reverse direction.
- [Foundry](https://book.getfoundry.sh/getting-started/installation),
  specifically `cast`, on your `$PATH`.
- `curl` and `python3` for the deposit status helper.
- A working `miden-client` install (see
  [CLI setup](../cli/index.md)).
- For the Miden → Sepolia direction only: use the Bali Miden bridge
  account and Miden ETH faucet account listed above.

## Direction 1: Sepolia to Miden (L1 to L2)

### 1. Initialise the client and create a destination wallet

If you have not already, initialise the client against the testnet and
create a wallet to receive the bridged funds:

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

Copy the 30-hex-character account ID - you will pass it as the bridge
destination in the next step.

This step also creates the local Miden client data directory used by the
reverse bridge flow. With the default global configuration, that
directory is `~/.miden`.

### 2. Create and fund a Sepolia keystore wallet

Create a Foundry keystore:

```sh
KEYSTORE_DIR=./
ACCOUNT_NAME=miden-bali-sepolia
cast wallet new "$KEYSTORE_DIR" "$ACCOUNT_NAME"
```

Use a Sepolia faucet (e.g. https://cloud.google.com/application/web3/faucet/ethereum/sepolia) to send test ETH to the new address so you can make a deposit to the L1 bridge deposit contract.

### 3. Download the helpers

The `bali-l1-deposit.sh` helper builds and broadcasts the
`bridgeAsset` transaction on Sepolia. The
`bali-l1-deposit-status.sh` helper checks the bridge service for the
latest deposit targeting your Miden account. The reverse-direction
`bali-l2-withdraw.sh` helper submits the B2AGG note on Miden. All
helpers read shared settings from `bali-bridge.conf`. They live
alongside this doc; download them and make them executable:

```sh
curl -O https://raw.githubusercontent.com/0xMiden/miden-client/main/docs/external/src/rust-client/get-started/bali-l1-deposit.sh
curl -O https://raw.githubusercontent.com/0xMiden/miden-client/main/docs/external/src/rust-client/get-started/bali-l1-deposit-status.sh
curl -O https://raw.githubusercontent.com/0xMiden/miden-client/main/docs/external/src/rust-client/get-started/bali-l2-withdraw.sh
curl -O https://raw.githubusercontent.com/0xMiden/miden-client/main/docs/external/src/rust-client/get-started/bali-bridge.conf.example
chmod +x bali-l1-deposit.sh bali-l1-deposit-status.sh bali-l2-withdraw.sh
cp bali-bridge.conf.example bali-bridge.conf
```

Edit `bali-bridge.conf` and fill in the Miden account ID you created in step 1:

```sh
MIDEN_ACCOUNT_ID=<account-id-from-step-1>
```

The config file also contains other bridge constants, so edit the file
for every value you want to change.

By default the deposit helper runs in `DRY_RUN=1` mode: it prints the
exact `cast send` command it would issue without broadcasting. Use this
to sanity-check inputs before spending gas. `DRY_RUN` is intentionally
not part of `bali-bridge.conf`; pass it only for the command you are
running.

### 4. Broadcast the deposit

After checking the dry run output, pass `DRY_RUN=0` for the real
broadcast:

```sh
DRY_RUN=0 ./bali-l1-deposit.sh
```

The script zero-pads the 15-byte Miden account ID into the 20-byte
slot the bridge contract expects, builds the `bridgeAsset` calldata,
and broadcasts via `cast send`. On success it prints the Sepolia L1
transaction hash.

### 5. Wait for the agglayer to issue the claim note

Budget roughly 15 minutes from broadcast. Sepolia finality takes
about 6 minutes before the agglayer can act; claim creation and
submission to Miden are fast.

To check the latest deposit for your destination account, run:

```sh
./bali-l1-deposit-status.sh
```

If `ready_for_claim=false`, wait a bit and run the status helper again.

### 6. Consume the claim note

Sync the client and consume the note targeted at your Miden account:

```sh
miden-client sync
miden-client consume-notes
```

### Decimal scaling

Sepolia ETH has 18 decimals; the Miden ETH faucet has 8 decimals
(scale factor of 10 between them). So `0.001 ETH` deposited on Sepolia
lands as `100_000` Miden-ETH units in your wallet.

## Direction 2: Miden to Sepolia (L2 to L1)

The reverse direction uses `bridge-out-tool` from
[gateway-fm/miden-agglayer](https://github.com/gateway-fm/miden-agglayer/tree/main)
to submit a Bridge-to-Agglayer (B2AGG) note on Miden. The helper script
in this guide reads `bali-bridge.conf` and passes the required
arguments to `bridge-out-tool` for you.

### 1. Build `bridge-out-tool`

Clone `miden-agglayer` and build the binary:

```sh
git clone https://github.com/gateway-fm/miden-agglayer.git
cd miden-agglayer
cargo build --release --bin bridge-out-tool
```

Add the binary to your `PATH`.

### 2. Fill in the reverse-direction config

Edit `bali-bridge.conf` and set:

```sh
MIDEN_STORE_DIR=~/.miden
ETH_ACCOUNT_ID=<your-sepolia-address>
MIDEN_WITHDRAW_AMOUNT=10000
```

The helper uses the same `MIDEN_ACCOUNT_ID` from the L1-to-L2 setup. `MIDEN_STORE_DIR` is the Miden client
data directory created by `miden-client init --network testnet` earlier. If you initialized the client with `MIDEN_CLIENT_HOME` or a
local `.miden` directory, set `MIDEN_STORE_DIR` to that directory instead.

`10000` Miden-ETH units = `0.0001 ETH` on L1.

### 3. Submit the B2AGG note

Run the helper once without setting `DRY_RUN` to inspect the exact
command:

```sh
./bali-l2-withdraw.sh
```

It prints the `bridge-out-tool` invocation it will run:

```sh
bridge-out-tool \
  --store-dir ~/.miden \
  --node-url https://rpc.testnet.miden.io:443 \
  --wallet-id <account-id-from-step-1> \
  --bridge-id mtst1aqn4y5pyessyw5rukd0wnmgq6srmn7np \
  --faucet-id mtst1az5g5k0tj7vsqcrp90z2dshsmskhyely \
  --amount 10000 \
  --dest-address <your-sepolia-address> \
  --dest-network 0
```

After checking the dry run output, pass `DRY_RUN=0` to submit the
B2AGG note:

```sh
DRY_RUN=0 ./bali-l2-withdraw.sh
```

The agglayer indexes the consumed note and emits a synthetic
`BridgeEvent`, typically within 30 seconds on a healthy node.

### 4. Wait for the certificate to settle

AggLayer settles certificates on a once-per-hour cadence, and
aggsender builds the certificate at the 50%-of-epoch mark, so
broadcast-to-claimable is usually 30 to 90 minutes depending on where
in the epoch you submit.

Poll the bridge service for `ready_for_claim=true`:

```sh
curl https://miden-testnet-bridge.dev.eu-north-3.gateway.fm/api/bridges/<ETH_ACCOUNT_ID>
```

Once ready, fetch the merkle proof and call `claimAsset` on the
Sepolia bridge contract with `cast send`. The reference script in
`miden-agglayer` (`scripts/e2e-l2-to-l1.sh`) shows the calldata
construction.

### Verification

- L2: your wallet balance decreases by the bridged amount after the
  B2AGG note is consumed (`miden-client account --show <id>`).
- L1: a `ClaimEvent` log appears on the bridge contract and your
  destination address balance increases by `amount - gas`.
- The Etherscan link to the `claimAsset` transaction is the
  authoritative confirmation.

## Troubleshooting

- Certificates occasionally land in `InError` state with a
  settler-side nonce collision. These recover automatically; keep
  polling rather than re-broadcasting. If a cert is stuck for more
  than about an hour, [open an
  issue](https://github.com/0xMiden/miden-client/issues) or contact
  the Miden team.
- "Sender has zero Sepolia balance" warnings from `bali-l1-deposit.sh`
  mean the script could reach the RPC but your funded EOA has no
  balance there. Either fund the address or fix the
  `ETH_KEYSTORE` path.
