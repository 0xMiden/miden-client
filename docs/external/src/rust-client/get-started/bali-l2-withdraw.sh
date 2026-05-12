#!/usr/bin/env bash
#
# Bali L2->L1 withdrawal helper for the Sepolia <-> Miden testnet bridge.
#
# Submits a Bridge-to-Agglayer note on Miden for withdrawal to a Sepolia
# address. Reads all settings from ./bali-bridge.conf.

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

require_config MIDEN_STORE_DIR
require_config MIDEN_NODE_URL
require_config MIDEN_ACCOUNT_ID
require_config MIDEN_BRIDGE_ID
require_config MIDEN_FAUCET_ID
require_config MIDEN_WITHDRAW_AMOUNT
require_config ETH_ACCOUNT_ID
require_config DEST_L1_NETWORK

BRIDGE_OUT_TOOL="${BRIDGE_OUT_TOOL:-bridge-out-tool}"
DRY_RUN="${DRY_RUN:-1}"

if [[ ! "$ETH_ACCOUNT_ID" =~ ^0x[0-9a-fA-F]{40}$ ]]; then
    echo "ERROR: ETH_ACCOUNT_ID must be a 20-byte hex address (0x + 40 hex chars)" >&2
    exit 1
fi

if [[ "$DRY_RUN" == "0" ]] && ! command -v "$BRIDGE_OUT_TOOL" >/dev/null 2>&1; then
    echo "ERROR: bridge-out-tool not found at '$BRIDGE_OUT_TOOL'" >&2
    echo "Set BRIDGE_OUT_TOOL in $config_file or add the binary to PATH." >&2
    exit 1
fi

CMD=(
    "$BRIDGE_OUT_TOOL"
    --store-dir "$MIDEN_STORE_DIR"
    --node-url "$MIDEN_NODE_URL"
    --wallet-id "$MIDEN_ACCOUNT_ID"
    --bridge-id "$MIDEN_BRIDGE_ID"
    --faucet-id "$MIDEN_FAUCET_ID"
    --amount "$MIDEN_WITHDRAW_AMOUNT"
    --dest-address "$ETH_ACCOUNT_ID"
    --dest-network "$DEST_L1_NETWORK"
)

cat <<EOF
=== Bali L2->L1 withdrawal (Miden -> Sepolia) ===
Miden store           : $MIDEN_STORE_DIR
Miden RPC             : $MIDEN_NODE_URL
Wallet ID             : $MIDEN_ACCOUNT_ID
Bridge account        : $MIDEN_BRIDGE_ID
Faucet account        : $MIDEN_FAUCET_ID
Destination L1        : $ETH_ACCOUNT_ID
Destination network   : $DEST_L1_NETWORK
Amount                : $MIDEN_WITHDRAW_AMOUNT Miden-ETH units
bridge-out-tool       : $BRIDGE_OUT_TOOL
DRY_RUN               : $DRY_RUN
EOF

echo
echo "Would run:"
printf '  %s \\\n    ' "${CMD[@]:0:1}"
printf '%s \\\n    ' "${CMD[@]:1}"
echo

if [[ "$DRY_RUN" != "0" ]]; then
    echo "DRY_RUN=1 - stopping here. Re-run with DRY_RUN=0 to submit the B2AGG note."
    exit 0
fi

echo "Submitting B2AGG note..."
"${CMD[@]}"
