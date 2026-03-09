# Wallet Example

Barebones React SDK wallet demo: create account, show balances, claim notes, send tokens.

## Run

```bash
cd ../..
yarn install
yarn build
cd examples/wallet
yarn install
yarn dev
```

## Key Recovery Warning

This example uses passkey encryption (`passkeyEncryption: true`) to protect account
keys at rest. The passkey (Touch ID / Face ID / Windows Hello) is the **sole** mechanism
guarding access to the encrypted keys in IndexedDB. If the passkey is lost or the
credential is deleted, the keys become **permanently unrecoverable**.

A production wallet should implement one or more recovery strategies:

- **Seed phrase backup** — provide a deterministic `initSeed` when creating the wallet
  and derive a mnemonic the user can write down. The same seed regenerates the same keys
  regardless of the passkey. Note: for **private accounts** (where account state is stored
  only locally, not on-chain), recovering the keys alone is not enough — you must also
  back up the account data (e.g. via `exportAccountFile`).
- **Account file export** — use `exportAccountFile()` while the passkey session is active
  to create a portable backup containing both the account state and secret keys.
- **External signer** — delegate key management to an external service (e.g. Turnkey,
  Para) that provides its own recovery flow.
