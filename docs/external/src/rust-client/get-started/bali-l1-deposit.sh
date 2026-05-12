#!/usr/bin/env bash
#
# Bali L1->L2 deposit helper for the Sepolia <-> Miden testnet bridge.
#
# Builds and (optionally) broadcasts a `bridgeAsset` transaction that
# locks ETH on the Sepolia bridge contract for delivery to a Miden
# account on rollup 73.
#
# Reads all settings from ./bali-bridge.conf.

set -euo pipefail

config_file="./bali-bridge.conf"

if [[ ! -f "$config_file" ]]; then
    echo "ERROR: missing $config_file. Copy bali-bridge.conf.example and fill it in." >&2
    exit 1
fi

# shellcheck source=/dev/null
source "$config_file"

require_config() {
    local key="$1"
    if [[ -z "${!key:-}" ]]; then
        echo "ERROR: $key must be set in $config_file" >&2
        exit 1
    fi
}

require_config SEPOLIA_RPC_URL
require_config ETH_KEYSTORE
require_config BRIDGE_ADDRESS
require_config DEST_NETWORK
require_config GAS_TOKEN_ADDRESS
require_config FORCE_UPDATE_GLOBAL_EXIT_ROOT
require_config GAS_LIMIT
require_config AMOUNT_ETH

DRY_RUN="${DRY_RUN:-1}"

WALLET_ARGS=(--keystore "$ETH_KEYSTORE")
if [[ -n "${ETH_KEYSTORE_PASSWORD_FILE:-}" ]]; then
    WALLET_ARGS+=(--password-file "$ETH_KEYSTORE_PASSWORD_FILE")
fi

# Derive sender address from the keystore for logging / default-destination.
FROM_ADDRESS="$(cast wallet address "${WALLET_ARGS[@]}")"

# Destination: either a Miden AccountId (15-byte / 30-hex) mapped into a 20-byte
# eth slot by padding 4 leading zero bytes + 1 trailing zero byte, or fall back
# to the sender's own address (dev-usage).
miden_to_eth() {
    local miden_addr="${1#0x}"
    if [[ ${#miden_addr} -ne 30 ]]; then
        echo "ERROR: MIDEN_ACCOUNT_ID must be 30 hex chars (15 bytes), got ${#miden_addr}" >&2
        exit 1
    fi
    echo "0x00000000${miden_addr}00"
}

if [[ -n "${MIDEN_ACCOUNT_ID:-}" ]]; then
    DEST_ADDRESS="$(miden_to_eth "$MIDEN_ACCOUNT_ID")"
    DEST_LABEL="$MIDEN_ACCOUNT_ID (Miden) -> $DEST_ADDRESS (Eth)"
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
    "$GAS_TOKEN_ADDRESS" \
    "$FORCE_UPDATE_GLOBAL_EXIT_ROOT" \
    "0x")"

# --- Pre-flight report ------------------------------------------------------
cat <<EOF
=== Bali L1->L2 deposit (Sepolia -> Miden rollup 73) ===
L1 RPC                : $SEPOLIA_RPC_URL
From                  : $FROM_ADDRESS
Keystore              : $ETH_KEYSTORE
Destination           : $DEST_LABEL
Bridge contract       : $BRIDGE_ADDRESS
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
    --gas-limit "$GAS_LIMIT"
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
    --gas-limit "$GAS_LIMIT" \
    --json)"
TX_HASH="$(echo "$RESULT" | jq -r '.transactionHash // empty')"
echo "tx: $TX_HASH"
cast receipt "$TX_HASH" --rpc-url "$SEPOLIA_RPC_URL" --json | jq '{status, blockNumber, logs: (.logs | length)}'
