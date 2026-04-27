# Miden Private Auction App Plan

## Goals
- Build a medium‑advanced demo that highlights Miden privacy while still showing public, understandable state.
- Support two auction modes:
  - **Sealed‑bid (private bids):** bids are hidden until reveal.
  - **Public highest bid (private bidder):** highest bid amount is public at all times, bidder identity stays private.
- Keep contracts in separate files: **one MASM per contract**.
- Provide a fully scripted deployment and demo flow.
- Use `packages/react-sdk` for the frontend.

## High‑Level Architecture
- **On‑chain accounts (MASM contracts)**
  - `auction_registry.masm` — creates auctions, stores metadata, tracks status.
  - `auction_instance.masm` — one per auction (or a param‑driven instance), holds state and enforces rules.
  - `escrow_vault.masm` — holds assets during bidding and settlement.
  - `reveal_verifier.masm` — verifies reveal proofs for sealed‑bid mode.
  - `settlement.masm` — finalizes auction, distributes funds/asset.
- **Off‑chain tooling**
  - CLI scripts for build, deploy, seed, and demo flows.
  - Minimal indexer script for UI (poll RPC + local cache).
- **Frontend**
  - React app using `packages/react-sdk` hooks (`useSend`, `useMultiSend`, `useTransaction`, `useTransactionHistory`, `useWaitForCommit`, `useWaitForNotes`, `useInternalTransfer`).

## Auction Modes

### 1) Sealed‑Bid Auction (Commit / Reveal)
- **Commit phase**
  - Bidder creates a commitment `C = H(bid_amount || nonce || bidder_account_id || auction_id)`.
  - Bidder sends P2ID note to `escrow_vault` with:
    - amount = bid_amount
    - note metadata includes `C` and `auction_id`
  - `auction_instance` stores commitment list + phase timeline.
- **Reveal phase**
  - Bidder submits `(bid_amount, nonce, bidder_account_id)`.
  - `reveal_verifier` checks commitment and presence of escrow note.
  - `auction_instance` updates current highest bid and highest bidder ID (kept private; only commitment or alias on chain).
- **Settlement**
  - Highest bidder gets the auctioned asset.
  - Others can withdraw from escrow (or auto‑refund).

### 2) Public Highest Bid, Private Bidder
- `auction_instance` keeps `highest_bid_amount` as public state.
- Bids are submitted via private notes to escrow; bidder identity stays private.
- Contract updates `highest_bid_amount` and stores a commitment for later settlement.

## Contracts (One MASM per file)
- `contracts/auction_registry.masm`
  - create auction, set parameters, provide auction IDs.
- `contracts/auction_instance.masm`
  - enforce phase timings, record commitments, store public highest bid (if enabled).
- `contracts/escrow_vault.masm`
  - receives bid notes; supports refund/withdrawal.
- `contracts/reveal_verifier.masm`
  - validates reveal and connects to escrow note inclusion.
- `contracts/settlement.masm`
  - final distribution logic (asset transfer to winner, refunds).

## State Model
- Auction metadata: asset_id, min_bid, start_time, commit_deadline, reveal_deadline, mode.
- Commitment records: list or map keyed by bidder ID or commitment hash.
- Public fields: highest_bid_amount, total_bids (optional), status.
- Private fields: bidder mapping or commitment list.

## Frontend UX Flows
- **Create auction**
  - Form to choose asset, auction mode, time windows, min bid.
  - Generates auction account and registers via `auction_registry`.
- **Place bid**
  - Sealed‑bid: client generates nonce, computes commitment, sends P2ID note to escrow, submits commitment.
  - Public highest bid: just send bid note; contract updates highest bid.
- **Reveal** (sealed‑bid)
  - Submit reveal params, update auction state.
- **Settle**
  - Owner or anyone triggers settlement after reveal deadline.
- **Withdraw**
  - Non‑winning bidders withdraw escrowed funds.

## React SDK Usage (Expected)
- Account load/import: `useImportAccount`.
- Send notes to escrow: `useSend` (single) or `useMultiSend` (batch).
- Internal moves between owned accounts: `useInternalTransfer`.
- Track tx status: `useTransaction` / `useTransactionHistory`.
- Wait for commit/notes: `useWaitForCommit` / `useWaitForNotes`.

## Scripts / Deployment (Fully Scripted)
- `scripts/auction/build.sh`
  - builds MASM contracts and bundles output.
- `scripts/auction/deploy.sh`
  - creates accounts for registry, escrow, instance, settlement.
  - uses `miden-client` to create accounts with packages.
  - publishes on‑chain by sending a setup transaction.
- `scripts/auction/seed.sh`
  - creates test accounts and mints demo assets.
- `scripts/auction/demo.sh`
  - runs a full end‑to‑end demo flow: create auction -> bid -> reveal -> settle -> withdraw.

## Data + API Layer
- Minimal indexing layer:
  - Poll `rpcUrl` for account state.
  - Cache in local storage or small local DB.
  - Optional: server‑side indexer for production demo.

## Milestones
1) Define MASM contract interfaces and storage layout.
2) Build `auction_registry` + `auction_instance` + `escrow_vault` minimal flows.
3) Add reveal/settlement contracts.
4) Scripted deployment + seed demo.
5) React frontend flows.
6) Polish UX and add docs.

## Open Decisions
- How to represent bidder identity on‑chain while preserving privacy.
- Whether refunds are push‑based or pull‑based.
- Whether public highest bid mode is default or optional.
- Auction instance model: one account per auction vs a single multi‑auction account.

