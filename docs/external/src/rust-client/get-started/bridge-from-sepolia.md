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
| Sepolia public RPC (fallback) | `https://ethereum-sepolia-rpc.publicnode.com` |

## Prerequisites

- A funded Sepolia EOA. The private key is used both to broadcast the
  L1 deposit and to pay gas on `claimAsset` in the reverse direction.
- [Foundry](https://book.getfoundry.sh/getting-started/installation),
  specifically `cast`, on your `$PATH`.
- A working `miden-client` install (see
  [CLI setup](../cli/index.md)).
- For the Miden → Sepolia direction only: the Miden bridge account ID
  and Miden ETH faucet ID for Bali. These are not yet exposed on a
  public endpoint - request them from the Miden team.

:::note Version compatibility
The deployed Bali agglayer is pinned to a specific `miden-protocol`
release. If you build `miden-client` from the latest `main`, account
authentication may use a variant the deployed agglayer does not yet
accept. If you encounter signature or auth errors during a claim,
contact the Miden team for a known-compatible client version.
:::

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

### 2. Download the L1 deposit helper

The `bali-l1-deposit.sh` helper builds and broadcasts the
`bridgeAsset` transaction on Sepolia. It lives alongside this doc;
download it and make it executable:

```sh
curl -O https://raw.githubusercontent.com/0xMiden/miden-client/main/docs/external/src/rust-client/get-started/bali-l1-deposit.sh
chmod +x bali-l1-deposit.sh
```

By default the script runs in `DRY_RUN=1` mode: it prints the exact
`cast send` command it would issue without broadcasting. Use this to
sanity-check inputs before spending gas.

### 3. Broadcast the deposit

```sh
DEST_MIDEN=<account-id-from-step-1> \
AMOUNT_ETH=0.001 \
SEPOLIA_PRIVATE_KEY=<your-key> \
SEPOLIA_RPC_URL=https://ethereum-sepolia-rpc.publicnode.com \
DRY_RUN=0 \
./bali-l1-deposit.sh
```

The script zero-pads the 15-byte Miden account ID into the 20-byte
slot the bridge contract expects, builds the `bridgeAsset` calldata,
and broadcasts via `cast send`. On success it prints the L1
transaction hash; save it for reference.

:::tip
Run once with the default `DRY_RUN=1` first - the printed `cast`
command shows you exactly what will be signed, and the script also
performs a balance check against `SEPOLIA_RPC_URL`.
:::

### 4. Wait for the agglayer to issue the claim note

Budget roughly 15 minutes from broadcast. Sepolia finality takes
about 6 minutes before the agglayer can act; claim creation and
submission to Miden are fast.

### 5. Consume the claim note

Sync the client and consume the note targeted at your wallet:

```sh
miden-client sync
miden-client consume-notes
```

With no arguments, `consume-notes` discovers any notes consumable by
the default account and consumes them. If you have multiple accounts,
target one explicitly:

```sh
miden-client consume-notes --account <account-id> <note-id>
```

Look up the note ID first with:

```sh
miden-client notes
```

### Decimal scaling

Sepolia ETH has 18 decimals; the Miden ETH faucet has 8 decimals
(scale factor of 10 between them). So `0.001 ETH` deposited on Sepolia
lands as `100_000` Miden-ETH units in your wallet.

## Direction 2: Miden to Sepolia (L2 to L1)

The reverse direction requires the `bridge-out-tool` binary and a
reference `claimAsset` script. Both live in
[gateway-fm/miden-agglayer](https://github.com/gateway-fm/miden-agglayer/tree/main)
(see `scripts/e2e-l2-to-l1.sh`). Build instructions are in that
repository's README.

The high-level flow is:

1. Submit a Bridge-to-Agglayer (B2AGG) note on Miden via
   `bridge-out-tool`, providing your Miden wallet ID, the bridge ID,
   the faucet ID, the destination L1 address, and an amount in
   Miden-ETH units (recall the 8-decimal scaling - `10000` units =
   `0.0001` ETH on L1).
2. The agglayer indexes the consumed note and emits a synthetic
   `BridgeEvent` (typically within 30 seconds on a healthy node).
3. Wait for the aggsender certificate to settle on L1. AggLayer
   settles certificates on a once-per-hour cadence, and aggsender
   builds the cert at the 50%-of-epoch mark, so broadcast-to-claimable
   is usually 30 to 90 minutes depending on where in the epoch you
   submit.
4. Poll the bridge service for `ready_for_claim=true`:

   ```sh
   curl https://miden-testnet-bridge.dev.eu-north-3.gateway.fm/api/bridges/<DEST_L1_ADDRESS>
   ```

5. Once ready, fetch the merkle proof and call `claimAsset` on the
   Sepolia bridge contract with `cast send`. The reference script in
   `gateway-fm/miden-agglayer` shows the calldata construction.

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
  `SEPOLIA_PRIVATE_KEY`.
- The destination address must be the 15-byte Miden account ID (30
  hex chars). Other formats are rejected by the helper script before
  any transaction is built.
