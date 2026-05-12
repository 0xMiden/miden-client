#!/usr/bin/env bash
#
# Bali L1->L2 deposit helper for the Sepolia <-> Miden testnet bridge.
#
# Builds and (optionally) broadcasts a `bridgeAsset` transaction that
# locks ETH on the Sepolia bridge contract for delivery to a Miden
# account on rollup 73.
#
# Bali constants (from the deployment manifest):
#   polygonZkEVMBridgeAddress  = 0x1348947e282138d8f377b467f7d9c2eb0f335d1f
#   rollupID / DEST_NETWORK    = 73
#   l2ChainID                  = 1259691107
#   gasTokenAddress            = 0x0 (ETH)
#
# Defaults to DRY_RUN=1: prints the exact `cast send` command but does
# NOT broadcast it. To actually send, re-run with DRY_RUN=0.
#
# Env vars:
#   SEPOLIA_RPC_URL                 (required) Infura/Alchemy/etc; needs eth_sendRawTransaction
#   SEPOLIA_KEYSTORE                (required) Path to a Foundry keystore file, funded with Sepolia ETH
#   SEPOLIA_KEYSTORE_PASSWORD_FILE  (optional) Path to the keystore password file. If unset, cast prompts.
#   DEST_MIDEN                      (optional) 0x<30-hex-chars> Miden AccountId to target.
#                                              Defaults to the sender's own Eth address (dev usage).
#   AMOUNT_ETH                      (optional) amount in ETH (default 0.001 for a first test)
#   DRY_RUN                         (optional) 1 to print only, 0 to broadcast (default 1)

set -euo pipefail

# --- Bali constants (from deployment manifest) ------------------------------
BRIDGE_ADDRESS="0x1348947e282138d8f377b467f7d9c2eb0f335d1f"
GLOBAL_EXIT_ROOT_ADDRESS="0x2968d6d736178f8fe7393cc33c87f29d9c287e78"
ROLLUP_MANAGER_ADDRESS="0xe2ef6215adc132df6913c8dd16487abf118d1764"
DEST_NETWORK=73
L2_CHAIN_ID=1259691107

# --- Sane first-test defaults ----------------------------------------------
AMOUNT_ETH="${AMOUNT_ETH:-0.001}"
DRY_RUN="${DRY_RUN:-1}"

# --- Required env vars ------------------------------------------------------
: "${SEPOLIA_RPC_URL:?SEPOLIA_RPC_URL must be set (Infura/Alchemy/etc)}"
: "${SEPOLIA_KEYSTORE:?SEPOLIA_KEYSTORE must be set (path to Foundry keystore file)}"

WALLET_ARGS=(--keystore "$SEPOLIA_KEYSTORE")
if [[ -n "${SEPOLIA_KEYSTORE_PASSWORD_FILE:-}" ]]; then
    WALLET_ARGS+=(--password-file "$SEPOLIA_KEYSTORE_PASSWORD_FILE")
fi

# Derive sender address from the keystore for logging / default-destination.
FROM_ADDRESS="$(cast wallet address "${WALLET_ARGS[@]}")"

# Destination: either a Miden AccountId (15-byte / 30-hex) mapped into a 20-byte
# eth slot by padding 4 leading zero bytes + 1 trailing zero byte, or fall back
# to the sender's own address (dev-usage).
miden_to_eth() {
    local miden_addr="${1#0x}"
    if [[ ${#miden_addr} -ne 30 ]]; then
        echo "ERROR: DEST_MIDEN must be 30 hex chars (15 bytes), got ${#miden_addr}" >&2
        exit 1
    fi
    echo "0x00000000${miden_addr}00"
}

if [[ -n "${DEST_MIDEN:-}" ]]; then
    DEST_ADDRESS="$(miden_to_eth "$DEST_MIDEN")"
    DEST_LABEL="$DEST_MIDEN (Miden) -> $DEST_ADDRESS (Eth)"
else
    DEST_ADDRESS="$FROM_ADDRESS"
    DEST_LABEL="$FROM_ADDRESS (sender)"
fi

# Convert ETH to wei without floating-point pitfalls
AMOUNT_WEI="$(cast --to-wei "$AMOUNT_ETH" eth)"

# Build the bridgeAsset calldata
#   bridgeAsset(uint32 destinationNetwork, address destinationAddress,
#               uint256 amount, address token, bool forceUpdateGlobalExitRoot,
#               bytes permitData)
CALLDATA="$(cast calldata 'bridgeAsset(uint32,address,uint256,address,bool,bytes)' \
    "$DEST_NETWORK" \
    "$DEST_ADDRESS" \
    "$AMOUNT_WEI" \
    "0x0000000000000000000000000000000000000000" \
    true \
    "0x")"

# --- Pre-flight report ------------------------------------------------------
cat <<EOF
=== Bali L1->L2 deposit (Sepolia -> Miden rollup 73) ===
L1 RPC                : $SEPOLIA_RPC_URL
From                  : $FROM_ADDRESS
Keystore              : $SEPOLIA_KEYSTORE
Destination           : $DEST_LABEL
Bridge contract       : $BRIDGE_ADDRESS
GlobalExitRoot (L1)   : $GLOBAL_EXIT_ROOT_ADDRESS
Amount                : $AMOUNT_ETH ETH ($AMOUNT_WEI wei)
DEST_NETWORK (rollup) : $DEST_NETWORK
DRY_RUN               : $DRY_RUN
EOF

# Balance check - free, non-destructive. Non-fatal on DRY_RUN so a wrong RPC URL
# still lets the operator inspect the cast command before fixing it.
if BALANCE_WEI="$(cast balance "$FROM_ADDRESS" --rpc-url "$SEPOLIA_RPC_URL" 2>/dev/null)"; then
    BALANCE_ETH="$(cast --from-wei "$BALANCE_WEI" eth)"
    echo "Balance               : $BALANCE_ETH ETH"
    if [[ "$BALANCE_WEI" == "0" ]]; then
        echo "WARN: sender has zero Sepolia balance - the real send would fail"
    fi
else
    echo "Balance               : <RPC unreachable - skipping check>"
    if [[ "$DRY_RUN" == "0" ]]; then
        echo "ERROR: cannot reach $SEPOLIA_RPC_URL. Fix RPC before re-running with DRY_RUN=0."
        exit 1
    fi
fi

# --- Print the cast command --------------------------------------------------
CMD=(
    cast send "$BRIDGE_ADDRESS"
    "$CALLDATA"
    --value "$AMOUNT_WEI"
    "${WALLET_ARGS[@]}"
    --rpc-url "$SEPOLIA_RPC_URL"
    --gas-limit 300000
)
echo
echo "Would run:"
printf '  %s \\\n    ' "${CMD[@]:0:2}"
printf '%s \\\n    ' "${CMD[@]:2}"
echo

if [[ "$DRY_RUN" != "0" ]]; then
    echo "DRY_RUN=1 - stopping here. Re-run with DRY_RUN=0 to actually broadcast."
    exit 0
fi

# --- Real send (only reached when DRY_RUN=0) --------------------------------
echo "Broadcasting deposit..."
RESULT="$(cast send "$BRIDGE_ADDRESS" \
    "$CALLDATA" \
    --value "$AMOUNT_WEI" \
    "${WALLET_ARGS[@]}" \
    --rpc-url "$SEPOLIA_RPC_URL" \
    --gas-limit 300000 \
    --json)"
TX_HASH="$(echo "$RESULT" | jq -r '.transactionHash // empty')"
echo "tx: $TX_HASH"
cast receipt "$TX_HASH" --rpc-url "$SEPOLIA_RPC_URL" --json | jq '{status, blockNumber, logs: (.logs | length)}'
