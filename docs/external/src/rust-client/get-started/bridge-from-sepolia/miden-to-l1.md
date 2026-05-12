---
title: Bridge from Miden to Sepolia
sidebar_position: 3
---

This flow submits a Bridge-to-Agglayer (B2AGG) note on Miden and later claims the
settled funds on Sepolia.

Complete [Setup](./setup.md) before starting this page.

## Build `bridge-out-tool`

The reverse direction uses `bridge-out-tool` from
[gateway-fm/miden-agglayer](https://github.com/gateway-fm/miden-agglayer/tree/main).

Clone `miden-agglayer` and build the binary:

```sh
git clone https://github.com/gateway-fm/miden-agglayer.git
cd miden-agglayer
cargo build --release --bin bridge-out-tool
```

Add the binary to your `PATH`.

## Fill in the withdrawal config

Edit `bali-bridge.conf` and set:

```sh
MIDEN_STORE_DIR=~/.miden
ETH_ACCOUNT_ID=<your-sepolia-address>
MIDEN_WITHDRAW_AMOUNT=10000
```

The helper uses the same `MIDEN_ACCOUNT_ID` from setup. `MIDEN_STORE_DIR` is the Miden
client data directory created by `miden-client init --network testnet` earlier. If you
initialized the client with `MIDEN_CLIENT_HOME` or a local `.miden` directory, set
`MIDEN_STORE_DIR` to that directory instead.

`10000` Miden-ETH units = `0.0001 ETH` on L1.

## Submit the B2AGG note

Run the helper once without setting `DRY_RUN` to inspect the exact command:

```sh
./bali-l2-withdraw.sh
```

It prints the `bridge-out-tool` invocation it will run:

```sh
bridge-out-tool \
  --store-dir ~/.miden \
  --node-url https://rpc.testnet.miden.io:443 \
  --wallet-id <account-id-from-setup> \
  --bridge-id mtst1aqn4y5pyessyw5rukd0wnmgq6srmn7np \
  --faucet-id mtst1az5g5k0tj7vsqcrp90z2dshsmskhyely \
  --amount 10000 \
  --dest-address <your-sepolia-address> \
  --dest-network 0
```

After checking the dry run output, pass `DRY_RUN=0` to submit the B2AGG note:

```sh
DRY_RUN=0 ./bali-l2-withdraw.sh
```

The agglayer indexes the consumed note and emits a synthetic `BridgeEvent`, typically
within 30 seconds on a healthy node.

## Wait for the certificate to settle

AggLayer settles certificates on a once-per-hour cadence, and aggsender builds the
certificate at the 50%-of-epoch mark, so broadcast-to-claimable is usually 30 to 90
minutes depending on where in the epoch you submit.

Poll the bridge service for `ready_for_claim=true`:

```sh
curl https://miden-testnet-bridge.dev.eu-north-3.gateway.fm/api/bridges/<ETH_ACCOUNT_ID>
```

Once ready, fetch the merkle proof and call `claimAsset` on the Sepolia bridge contract
with `cast send`. The reference script in `miden-agglayer`
(`scripts/e2e-l2-to-l1.sh`) shows the calldata construction.

## Verification

- L2: your wallet balance decreases by the bridged amount after the B2AGG note is
  consumed (`miden-client account --show <id>`).
- L1: a `ClaimEvent` log appears on the bridge contract and your destination address
  balance increases by `amount - gas`.
- The Etherscan link to the `claimAsset` transaction is the authoritative confirmation.

## Troubleshooting

- Certificates occasionally land in `InError` state with a settler-side nonce collision.
  These recover automatically; keep polling rather than re-broadcasting. If a cert is
  stuck for more than about an hour, [open an
  issue](https://github.com/0xMiden/miden-client/issues) or contact the Miden team.
