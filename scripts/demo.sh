#!/usr/bin/env bash
# ============================================================================
#  Miden Client — Interactive Demo
#  A story-driven walkthrough of the Miden rollup client for developers.
#
#  Prerequisites:
#    - A running Miden node (localhost:57291 by default)
#    - The miden-client binary in PATH or built in target/release/
#
#  Usage:
#    ./scripts/demo.sh              # Run the full demo
#    ./scripts/demo.sh --auto       # Auto-advance (no pauses between steps)
#    ./scripts/demo.sh --fast       # Skip sync waits (for pre-synced state)
# ============================================================================

set -euo pipefail

# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------

AUTO_MODE=false
FAST_MODE=false
for arg in "$@"; do
  case "$arg" in
    --auto) AUTO_MODE=true ;;
    --fast) FAST_MODE=true ;;
  esac
done

# Colors and formatting
BOLD='\033[1m'
DIM='\033[2m'
RESET='\033[0m'
CYAN='\033[36m'
GREEN='\033[32m'
YELLOW='\033[33m'
MAGENTA='\033[35m'
RED='\033[31m'
BLUE='\033[34m'
WHITE='\033[37m'
BG_BLUE='\033[44m'
BG_GREEN='\033[42m'
BG_MAGENTA='\033[45m'

# Temporary home for this demo (isolated from any real client state)
export MIDEN_CLIENT_HOME=$(mktemp -d)
DEMO_TMPDIR="$MIDEN_CLIENT_HOME"  # alias for clarity
trap 'rm -rf "$DEMO_TMPDIR"' EXIT

# Find the miden-client binary
if command -v miden-client &>/dev/null; then
  CLI="miden-client"
elif [[ -f "./target/release/miden-client" ]]; then
  CLI="./target/release/miden-client"
else
  echo -e "${RED}Error: miden-client not found. Build it first with 'make build'.${RESET}"
  exit 1
fi

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

step_number=0

banner() {
  echo ""
  echo -e "${BG_BLUE}${WHITE}${BOLD}                                                                ${RESET}"
  echo -e "${BG_BLUE}${WHITE}${BOLD}  $1$(printf '%*s' $((62 - ${#1})) '')${RESET}"
  echo -e "${BG_BLUE}${WHITE}${BOLD}                                                                ${RESET}"
  echo ""
}

step() {
  step_number=$((step_number + 1))
  echo ""
  echo -e "  ${MAGENTA}${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
  echo -e "  ${MAGENTA}${BOLD}  STEP $step_number: $1${RESET}"
  echo -e "  ${MAGENTA}${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
  echo ""
}

narrate() {
  echo -e "  ${DIM}$1${RESET}"
}

explain() {
  echo -e "  ${CYAN}💡 $1${RESET}"
}

success() {
  echo -e "  ${GREEN}✓ $1${RESET}"
}

warn() {
  echo -e "  ${YELLOW}⚠  $1${RESET}"
}

run_cmd() {
  local label="$1"
  shift
  echo ""
  echo -e "  ${YELLOW}▸ ${BOLD}$label${RESET}"
  echo -e "  ${DIM}\$ $*${RESET}"
  echo ""

  # Run and indent output
  "$@" 2>&1 | sed 's/^/    /' || true
  echo ""
}

pause() {
  if [[ "$AUTO_MODE" == false ]]; then
    echo ""
    echo -ne "  ${DIM}Press Enter to continue...${RESET}"
    read -r
  fi
}

# Extract account ID from "new-wallet" or "new-account" output.
# The CLI prints the account ID in a line like:
#   "Account ID: 0x..."  or in a table format.
# We'll capture the full output and grep for it.
extract_account_id() {
  echo "$1" | grep -oE '0x[0-9a-fA-F]{16}' | head -1
}

# ---------------------------------------------------------------------------
# Pre-flight checks
# ---------------------------------------------------------------------------

clear

cat << 'LOGO'

    ███╗   ███╗██╗██████╗ ███████╗███╗   ██╗
    ████╗ ████║██║██╔══██╗██╔════╝████╗  ██║
    ██╔████╔██║██║██║  ██║█████╗  ██╔██╗ ██║
    ██║╚██╔╝██║██║██║  ██║██╔══╝  ██║╚██╗██║
    ██║ ╚═╝ ██║██║██████╔╝███████╗██║ ╚████║
    ╚═╝     ╚═╝╚═╝╚═════╝ ╚══════╝╚═╝  ╚═══╝

LOGO

echo -e "    ${BOLD}The Miden Client — Interactive Demo${RESET}"
echo -e "    ${DIM}A zero-knowledge rollup with client-side proving${RESET}"
echo ""
echo -e "    ${DIM}Binary:  ${CLI}${RESET}"
echo -e "    ${DIM}Home:    ${MIDEN_CLIENT_HOME}${RESET}"
echo ""

# Pre-flight: check if node is reachable
echo -e "  ${DIM}Checking node connectivity (localhost:57291)...${RESET}"
if command -v grpcurl &>/dev/null; then
  if grpcurl -plaintext localhost:57291 rpc.Api/Status &>/dev/null 2>&1; then
    success "Miden node is running"
  else
    warn "Could not reach Miden node at localhost:57291"
    echo -e "  ${DIM}  Start one with: make start-node-background${RESET}"
    echo -e "  ${YELLOW}  Continuing anyway — commands will fail if the node isn't reachable.${RESET}"
  fi
else
  echo -e "  ${DIM}(grpcurl not found, skipping connectivity check)${RESET}"
fi

echo ""
echo -e "  ${BOLD}The Story${RESET}"
echo ""
echo -e "  ${WHITE}Alice runs a token faucet on the Miden network.${RESET}"
echo -e "  ${WHITE}She mints tokens and sends some to her friend Bob.${RESET}"
echo -e "  ${WHITE}All transactions are proven locally using zero-knowledge proofs —${RESET}"
echo -e "  ${WHITE}the network only verifies proofs, never sees private state.${RESET}"

pause

# ═══════════════════════════════════════════════════════════════════════════
# ACT 1: SETUP
# ═══════════════════════════════════════════════════════════════════════════

banner "ACT 1 — Setting the Stage"

step "Initialize the Client"

narrate "The Miden client stores account keys, transaction history, and a partial"
narrate "view of the blockchain locally. Let's initialize it for our local network."
echo ""
explain "This creates a config file and an empty SQLite store."
explain "Each client instance is self-contained — keys never leave the machine."

run_cmd "Initialize client for localhost" $CLI init --network localhost

success "Client initialized at $MIDEN_CLIENT_HOME"

pause

# ═══════════════════════════════════════════════════════════════════════════
# ACT 2: ACCOUNTS
# ═══════════════════════════════════════════════════════════════════════════

banner "ACT 2 — Creating Accounts"

step "Create Alice's Faucet"

narrate "A faucet is a special account type that can mint new tokens."
narrate "Alice will use this to create her own fungible token called 'ALI'."
echo ""
explain "Account types in Miden:"
explain "  • Regular Account — holds assets, runs smart contracts"
explain "  • Fungible Faucet — mints fungible tokens (like ERC-20)"
explain "  • Non-Fungible Faucet — mints unique tokens (like ERC-721)"
echo ""
explain "Storage modes:"
explain "  • Public — state visible on-chain (needed for faucets)"
explain "  • Private — state kept locally, only a commitment goes on-chain"
echo ""
explain "Creating a faucet requires specifying token metadata:"
explain "  • Ticker symbol, decimal precision, and max supply"
explain "  • This is provided via an init storage data file."

# Create init storage data for the faucet
FAUCET_INIT_DATA="${DEMO_TMPDIR}/faucet_init.toml"
cat > "$FAUCET_INIT_DATA" << 'TOML'
["miden::standards::fungible_faucets::metadata"]
decimals = "8"
max_supply = "1000000"
ticker = "ALI"
TOML

FAUCET_OUTPUT=$($CLI new-account \
  --account-type fungible-faucet \
  --storage-mode public \
  -p basic-fungible-faucet \
  -i "$FAUCET_INIT_DATA" \
  --deploy 2>&1) || true

echo -e "  ${YELLOW}▸ ${BOLD}Create a fungible faucet (public, deployed)${RESET}"
echo -e "  ${DIM}\$ miden-client new-account --account-type fungible-faucet -s public -p basic-fungible-faucet -i faucet_init.toml --deploy${RESET}"
echo ""
echo "$FAUCET_OUTPUT" | sed 's/^/    /'
echo ""

FAUCET_ID=$(extract_account_id "$FAUCET_OUTPUT")

if [[ -n "$FAUCET_ID" ]]; then
  success "Alice's faucet created: ${BOLD}$FAUCET_ID${RESET}"
else
  warn "Could not extract faucet ID from output. The demo may not work correctly."
  FAUCET_ID="FAUCET_ID_PLACEHOLDER"
fi

pause

step "Create Alice's Wallet"

narrate "Alice also needs a regular wallet to hold the tokens she mints."

ALICE_OUTPUT=$($CLI new-wallet --storage-mode private --deploy 2>&1) || true

echo -e "  ${YELLOW}▸ ${BOLD}Create Alice's wallet (private)${RESET}"
echo -e "  ${DIM}\$ miden-client new-wallet --storage-mode private --deploy${RESET}"
echo ""
echo "$ALICE_OUTPUT" | sed 's/^/    /'
echo ""

ALICE_ID=$(extract_account_id "$ALICE_OUTPUT")

if [[ -n "$ALICE_ID" ]]; then
  success "Alice's wallet created: ${BOLD}$ALICE_ID${RESET}"
else
  warn "Could not extract Alice's wallet ID."
  ALICE_ID="ALICE_ID_PLACEHOLDER"
fi

pause

step "Create Bob's Wallet"

narrate "Bob is Alice's friend. He also creates a wallet on the network."

BOB_OUTPUT=$($CLI new-wallet --storage-mode private --deploy 2>&1) || true

echo -e "  ${YELLOW}▸ ${BOLD}Create Bob's wallet (private)${RESET}"
echo -e "  ${DIM}\$ miden-client new-wallet --storage-mode private --deploy${RESET}"
echo ""
echo "$BOB_OUTPUT" | sed 's/^/    /'
echo ""

BOB_ID=$(extract_account_id "$BOB_OUTPUT")

if [[ -n "$BOB_ID" ]]; then
  success "Bob's wallet created: ${BOLD}$BOB_ID${RESET}"
else
  warn "Could not extract Bob's wallet ID."
  BOB_ID="BOB_ID_PLACEHOLDER"
fi

pause

step "View All Accounts"

narrate "Let's see what we've got. The client tracks all accounts we've created."

run_cmd "List all accounts" $CLI account --list

explain "Notice the storage modes: the faucet is ${BOLD}Public${RESET}${CYAN} (visible on-chain)"
explain "while the wallets are ${BOLD}Private${RESET}${CYAN} (only commitments on-chain)."

pause

# ═══════════════════════════════════════════════════════════════════════════
# ACT 3: SYNC
# ═══════════════════════════════════════════════════════════════════════════

banner "ACT 3 — Syncing with the Network"

step "Sync Client State"

narrate "The Miden client is a light client. It doesn't watch every block —"
narrate "instead, it periodically syncs to catch up with the network."
echo ""
explain "Sync fetches:"
explain "  • New block headers and Merkle proofs"
explain "  • Notes addressed to our accounts"
explain "  • State updates for tracked accounts"
explain "  • Nullifiers that mark consumed notes"

run_cmd "Sync with network" $CLI sync

success "Client is now up to date with the network."

pause

# ═══════════════════════════════════════════════════════════════════════════
# ACT 4: MINTING
# ═══════════════════════════════════════════════════════════════════════════

banner "ACT 4 — Minting Tokens"

step "Mint Tokens to Alice"

narrate "Alice uses her faucet to mint 1000 tokens to herself."
narrate "This creates a transaction that:"
narrate "  1. Executes locally against Alice's faucet account"
narrate "  2. Generates a zero-knowledge proof on her machine"
narrate "  3. Submits the proof to the network for verification"
echo ""
explain "The network never sees the faucet's internal state —"
explain "it only verifies that the proof is valid."

run_cmd "Mint 1000 tokens from faucet to Alice" \
  $CLI mint \
    --target "$ALICE_ID" \
    --asset "${FAUCET_ID}::1000" \
    --note-type public \
    --force

success "Mint transaction submitted!"
explain "The tokens are packaged in a ${BOLD}note${RESET}${CYAN} — Miden's version of a UTXO."
explain "Alice needs to consume this note to add the tokens to her wallet."

pause

step "Sync to Discover the Minted Note"

narrate "After the network processes the transaction, we sync to see the note."

run_cmd "Sync" $CLI sync

run_cmd "List consumable notes" $CLI notes --list consumable

explain "The note is now ${BOLD}committed${RESET}${CYAN} on-chain and ready to be consumed."

pause

step "Alice Consumes the Note"

narrate "Alice consumes the note to add the tokens to her wallet."
narrate "This is another transaction — proven locally, verified on-chain."

run_cmd "Consume all notes for Alice's wallet" \
  $CLI consume-notes \
    --account "$ALICE_ID" \
    --force

success "Note consumed! Tokens are now in Alice's wallet."

pause

step "Verify Alice's Balance"

narrate "Let's sync once more and check Alice's account."

run_cmd "Sync" $CLI sync

run_cmd "Show Alice's account details" $CLI account --show "$ALICE_ID"

success "Alice now holds 1000 tokens from her faucet!"

pause

# ═══════════════════════════════════════════════════════════════════════════
# ACT 5: TRANSFER
# ═══════════════════════════════════════════════════════════════════════════

banner "ACT 5 — Transferring Tokens"

step "Alice Sends 250 Tokens to Bob"

narrate "Alice wants to send 250 tokens to Bob."
narrate "She creates a Pay-to-ID (P2ID) transaction — the note is locked"
narrate "so that only Bob's account can consume it."
echo ""
explain "Note types:"
explain "  • Public — note data visible on-chain (anyone can see the transfer)"
explain "  • Private — only a commitment on-chain (Bob needs to discover it)"

run_cmd "Send 250 tokens from Alice to Bob" \
  $CLI send \
    --sender "$ALICE_ID" \
    --target "$BOB_ID" \
    --asset "${FAUCET_ID}::250" \
    --note-type public \
    --force

success "Transfer transaction submitted!"
explain "A P2ID note has been created, locked to Bob's account ID."

pause

step "Bob Receives the Tokens"

narrate "Bob syncs his client to discover the incoming note, then consumes it."

run_cmd "Sync to discover incoming note" $CLI sync

run_cmd "List Bob's consumable notes" $CLI notes --list consumable

run_cmd "Bob consumes the incoming note" \
  $CLI consume-notes \
    --account "$BOB_ID" \
    --force

success "Bob consumed the note!"

pause

step "Final Sync and Balance Check"

narrate "Let's do a final sync and check everyone's balances."

run_cmd "Final sync" $CLI sync

echo ""
echo -e "  ${BG_GREEN}${WHITE}${BOLD}  ALICE'S ACCOUNT  ${RESET}"
run_cmd "Alice's wallet" $CLI account --show "$ALICE_ID"

echo -e "  ${BG_MAGENTA}${WHITE}${BOLD}  BOB'S ACCOUNT    ${RESET}"
run_cmd "Bob's wallet" $CLI account --show "$BOB_ID"

echo -e "  ${BG_BLUE}${WHITE}${BOLD}  FAUCET ACCOUNT   ${RESET}"
run_cmd "Alice's faucet" $CLI account --show "$FAUCET_ID"

echo ""
echo -e "  ${YELLOW}▸ ${BOLD}Transaction History${RESET}"
run_cmd "All transactions" $CLI tx --list

pause

# ═══════════════════════════════════════════════════════════════════════════
# EPILOGUE
# ═══════════════════════════════════════════════════════════════════════════

banner "Demo Complete!"

echo -e "  ${BOLD}What just happened:${RESET}"
echo ""
echo -e "  ${GREEN}1.${RESET} Created 3 accounts (faucet + 2 wallets) on a local Miden network"
echo -e "  ${GREEN}2.${RESET} Minted 1000 fungible tokens from the faucet"
echo -e "  ${GREEN}3.${RESET} Transferred 250 tokens from Alice to Bob"
echo -e "  ${GREEN}4.${RESET} All transactions were ${BOLD}proven locally${RESET} using zero-knowledge proofs"
echo -e "  ${GREEN}5.${RESET} The network only verified proofs — it never saw private account state"
echo ""
echo -e "  ${BOLD}How it works under the hood:${RESET}"
echo ""
echo -e "  ${DIM}  ┌─────────────────────────────────┐      ┌────────────────────┐${RESET}"
echo -e "  ${DIM}  │         ${CYAN}Miden Client${DIM}              │      │   ${YELLOW}Miden Network${DIM}    │${RESET}"
echo -e "  ${DIM}  │                                 │      │                    │${RESET}"
echo -e "  ${DIM}  │  1. Build transaction request   │      │                    │${RESET}"
echo -e "  ${DIM}  │  2. Execute against local state  │      │                    │${RESET}"
echo -e "  ${DIM}  │  3. Generate ZK proof (STARK)   │      │                    │${RESET}"
echo -e "  ${DIM}  │  4. Submit proof ──────────────────────▶│ Verify proof       │${RESET}"
echo -e "  ${DIM}  │                                 │      │ Update state       │${RESET}"
echo -e "  ${DIM}  │  5. Sync ◀────────────────────────────│ Send updates       │${RESET}"
echo -e "  ${DIM}  │  6. Update local view           │      │                    │${RESET}"
echo -e "  ${DIM}  └─────────────────────────────────┘      └────────────────────┘${RESET}"
echo ""
echo -e "  ${BOLD}Key Concepts:${RESET}"
echo ""
echo -e "  ${CYAN}Accounts${RESET}  — Smart contract-like entities that hold assets and run code"
echo -e "  ${CYAN}Notes${RESET}     — UTXO-like containers that carry assets between accounts"
echo -e "  ${CYAN}Proofs${RESET}    — Zero-knowledge proofs generated client-side for every tx"
echo -e "  ${CYAN}Sync${RESET}      — Light client pattern: fetch only what matters to your accounts"
echo ""
echo -e "  ${BOLD}Expected final balances:${RESET}"
echo -e "  ${WHITE}  Alice's wallet:  750 tokens${RESET}"
echo -e "  ${WHITE}  Bob's wallet:    250 tokens${RESET}"
echo -e "  ${WHITE}  Faucet:          tracks total supply (1000 minted)${RESET}"
echo ""
echo -e "  ${DIM}Learn more: https://docs.miden.io${RESET}"
echo -e "  ${DIM}Source:     https://github.com/0xMiden/miden-client${RESET}"
echo ""
echo -e "  ${DIM}Temporary data was stored in: $MIDEN_CLIENT_HOME${RESET}"
echo -e "  ${DIM}It will be cleaned up automatically.${RESET}"
echo ""
