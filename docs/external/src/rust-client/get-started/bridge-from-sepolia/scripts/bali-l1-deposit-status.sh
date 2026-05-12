#!/usr/bin/env bash
#
# Bali L1->L2 deposit status helper for the Sepolia <-> Miden testnet bridge.
#
# Checks the bridge service for the latest Sepolia deposit targeting a Miden
# account on rollup 73. Reads all settings from ./bali-bridge.conf.
#
# Requires: curl, python3.

set -euo pipefail

config_file="./bali-bridge.conf"

if [[ ! -f "$config_file" ]]; then
    echo "ERROR: missing $config_file. Copy bali-bridge.conf.example and fill it in." >&2
    exit 1
fi

# shellcheck source=/dev/null
source "$config_file"

if [[ -z "${BRIDGE_SERVICE_API:-}" ]]; then
    echo "ERROR: BRIDGE_SERVICE_API must be set in $config_file" >&2
    exit 1
fi

miden_to_eth() {
    local miden_addr="${1#0x}"
    if [[ ! "$miden_addr" =~ ^[0-9a-fA-F]{30}$ ]]; then
        echo "ERROR: MIDEN_ACCOUNT_ID must be 30 hex chars (15 bytes), got ${#miden_addr}" >&2
        exit 1
    fi

    echo "0x00000000${miden_addr}00"
}

if [[ -n "${MIDEN_ACCOUNT_ID:-}" ]]; then
    DEST_ADDRESS="$(miden_to_eth "$MIDEN_ACCOUNT_ID")"
    DEST_LABEL="$MIDEN_ACCOUNT_ID (Miden) -> $DEST_ADDRESS (bridge destination)"
elif [[ -n "${bridge_destination_address:-}" ]]; then
    if [[ ! "$bridge_destination_address" =~ ^0x[0-9a-fA-F]{40}$ ]]; then
        echo "ERROR: bridge_destination_address must be a 20-byte hex address (0x + 40 hex chars)" >&2
        exit 1
    fi
    DEST_ADDRESS="$bridge_destination_address"
    DEST_LABEL="$bridge_destination_address"
else
    echo "ERROR: set MIDEN_ACCOUNT_ID or bridge_destination_address" >&2
    exit 1
fi

URL="${BRIDGE_SERVICE_API%/}/bridges/$DEST_ADDRESS?limit=1&offset=0"

print_status() {
    local response
    if ! response="$(curl -fsS "$URL")"; then
        echo "ERROR: failed to fetch bridge status from $URL" >&2
        return 2
    fi

    printf '%s' "$response" | python3 -c '
import json
import sys

try:
    data = json.load(sys.stdin)
except json.JSONDecodeError as err:
    print(f"ERROR: bridge service returned invalid JSON: {err}", file=sys.stderr)
    sys.exit(2)

deposits = data.get("deposits") or []

if not deposits:
    print("ready_for_claim=none deposits=0")
    sys.exit(1)

deposit = deposits[0]
ready = deposit.get("ready_for_claim")
fields = [
    "ready_for_claim={}".format(str(ready).lower()),
    "tx_hash={}".format(deposit.get("tx_hash", "")),
    "amount={}".format(deposit.get("amount", "")),
    "deposit_cnt={}".format(deposit.get("deposit_cnt", "")),
    "block_num={}".format(deposit.get("block_num", "")),
    "global_index={}".format(deposit.get("global_index", "")),
]
print(" ".join(fields))
sys.exit(0 if ready is True else 1)
'
}

cat <<EOF
=== Bali L1->L2 deposit status ===
Destination           : $DEST_LABEL
Bridge service        : $URL
EOF

set +e
print_status
status=$?
set -e

if [[ "$status" == "1" ]]; then
    exit 0
fi

exit "$status"
