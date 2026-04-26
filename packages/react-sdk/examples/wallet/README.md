# Wallet Example — Pattern B cross-signer test bench

Minimal React SDK wallet demo wired up against **all three** Miden signer
providers (MidenFi wallet adapter, Para, Turnkey). The dApp source code
(`src/App.tsx`) is signer-agnostic — it imports only from `@miden-sdk/react`
and works identically against whichever signer the user picks via
`MultiSignerProvider`.

## Pattern B branches under test

This example pulls each signer / SDK package from a local checkout of its
respective Pattern B branch. PRs (when opened) for the cross-repo work:

| Repo | Branch | Local path |
|---|---|---|
| miden-client (this repo) | `wiktor/signer-pattern-b` | `~/miden/miden-client` |
| miden-wallet-adapter | `wiktor/signer-pattern-b` | `~/miden/miden-wallet-adapter` |
| miden-wallet | `wiktor/signer-pattern-b` | `~/miden/miden-wallet` |
| miden-para | `wiktor/signer-pattern-b` | `~/miden/miden-para` |
| miden-turnkey | `wiktor/signer-pattern-b` | `~/miden/miden-turnkey` |

GitHub compare URLs (PRs not opened yet):

- https://github.com/0xMiden/miden-client/compare/main...wiktor/signer-pattern-b
- https://github.com/0xMiden/wallet-adapter/compare/main...wiktor/signer-pattern-b
- https://github.com/0xMiden/wallet/compare/main...wiktor/signer-pattern-b
- https://github.com/0xMiden/para-sdk/compare/main...wiktor/signer-pattern-b
- https://github.com/0xMiden/turnkey-sdk/compare/main...wiktor/signer-pattern-b

## Setup

### 1. Build all source packages

The example's `package.json` resolves the four signer/SDK packages via `file:`
deps pointing at sibling repos. Each must be built first.

```bash
# SDK
cd ~/miden/miden-client/packages/react-sdk && yarn build

# Wallet adapter (build packages individually — workspace build pulls in
# unrelated example projects that don't compile)
cd ~/miden/miden-wallet-adapter/packages/core/base && npx tsc
cd ~/miden/miden-wallet-adapter/packages/wallets/miden && npx tsc
cd ~/miden/miden-wallet-adapter/packages/core/react && npx tsc

# Para
cd ~/miden/miden-para && yarn build
cd ~/miden/miden-para/packages/use-miden-para-react && yarn build

# Turnkey
cd ~/miden/miden-turnkey && yarn build
cd ~/miden/miden-turnkey/packages/use-miden-turnkey-react && yarn build
```

### 2. Install + run the example

```bash
cd ~/miden/miden-client/packages/react-sdk/examples/wallet
yarn install   # installs file: deps from the built packages above
yarn dev       # → http://localhost:5173/
```

### 3. Configure signer credentials

Copy `.env.example` → `.env` and fill in:

```
VITE_PARA_API_KEY=...    # from getpara.com (BETA env)
VITE_TURNKEY_ORG_ID=...  # from your Turnkey dashboard
```

For MidenFi: load the wallet's `dist/chrome_unpacked/` as an unpacked Chrome
extension (Settings → Extensions → Developer mode → Load unpacked). Build
the wallet first with `yarn build:chrome` from `~/miden/miden-wallet`.

### 4. What to verify

The example's `App.tsx` uses only universal `@miden-sdk/react` hooks:
`useMiden`, `useSigner`, `useMultiSigner`, `useAccounts`, `useAccount`,
`useNotes`, `useCreateWallet`, `useConsume`, `useSend`. **No adapter-
specific imports.**

For each of MidenFi / Para / Turnkey:

1. Connect via the SignerSelector.
2. Confirm the wallet UI loads (account, balances, notes).
3. **Pattern B read path**: send a private note to the connected account
   from another wallet. Within ~15 seconds (auto-sync interval), it should
   appear in the "Unclaimed notes" panel — that's `ingestState` backfilling
   the dApp's local store.
4. **Pattern B write path**: clicking "Claim" on the private note should
   succeed via the universal `useConsume` hook.
5. **`useSignBytes`**: there's no UI for this in the current example; if
   you want to exercise it manually, open the browser console and run
   `window.__SIGNER__.signBytes(new Uint8Array(32), 'word')` (after
   exposing the signer via a small dev hook — TBD if added).

Same `App.tsx` byte-for-byte against all three signers is the canonical
proof of Pattern B's "1-line provider swap" pitch.

## Notes on the local-link setup

- The vite config aliases `@miden-sdk/react` to the local source
  (`../../src/index.ts`) so you don't need to rebuild the SDK between
  edits — vite picks up source changes via HMR.
- The other adapter packages need a rebuild (`yarn build` in their
  package dir) to pick up changes; the example's `node_modules/` is
  populated via `file:` copy at install time.
- Para's `@getpara/react-core` does dynamic imports for many optional
  AA / chain-integration packages. The vite config externalizes those
  via a regex so the build doesn't fail on missing optional deps —
  runtime usage of those features will throw, but our App.tsx never
  reaches them.
- `@miden-sdk/miden-sdk` registry version 0.14.4 is missing
  `resolveAuthScheme` (added in HEAD, not yet released). The SDK's
  `useCreateWallet` / `useCreateFaucet` lazy-import it; the registry
  copy works for everything except the create-wallet/faucet paths,
  which throw a clear error when invoked.
