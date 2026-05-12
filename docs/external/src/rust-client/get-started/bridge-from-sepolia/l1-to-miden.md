---
title: Bridge from Sepolia to Miden
sidebar_position: 2
---

This flow locks ETH on Sepolia and issues a claim note for your Miden account on the
Bali testnet.

Complete [Setup](./setup.md) before starting this page.

## Broadcast the deposit

Run the helper once without setting `DRY_RUN` to inspect the exact command:

```sh
./bali-l1-deposit.sh
```

After checking the dry run output, pass `DRY_RUN=0` for the real broadcast:

```sh
DRY_RUN=0 ./bali-l1-deposit.sh
```

The script zero-pads the 15-byte Miden account ID into the 20-byte slot the bridge
contract expects, builds the `bridgeAsset` calldata, and broadcasts via `cast send`. On
success it prints the Sepolia L1 transaction hash.

## Wait for the agglayer to issue the claim note

Budget roughly 15 minutes from broadcast. Sepolia finality takes about 6 minutes before
the agglayer can act; claim creation and submission to Miden are fast.

To check the latest deposit for your destination account, run:

```sh
./bali-l1-deposit-status.sh
```

If `ready_for_claim=false`, wait a bit and run the status helper again.

## Consume the claim note

Sync the client and consume the note targeted at your Miden account:

```sh
miden-client sync
miden-client consume-notes
```

## Decimal scaling

Sepolia ETH has 18 decimals; the Miden ETH faucet has 8 decimals (scale factor of 10
between them). So `0.001 ETH` deposited on Sepolia lands as `100_000` Miden-ETH units
in your wallet.

## Troubleshooting

- "Sender has zero Sepolia balance" warnings from `bali-l1-deposit.sh` mean the script
  could reach the RPC but your funded EOA has no balance there. Either fund the address
  or fix the `ETH_KEYSTORE` path.
