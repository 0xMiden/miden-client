# Wallet Example

Barebones React SDK wallet demo: create account, show balances, claim notes,
send tokens. Wired against three signer providers (MidenFi wallet adapter,
Para, Turnkey) under `MultiSignerProvider` so the same `App.tsx` runs against
any of them with no code changes.

## Run

```bash
cd ../..
yarn install
yarn build
cd examples/wallet
yarn install
yarn dev
```

Set `VITE_PARA_API_KEY` and `VITE_TURNKEY_ORG_ID` in a `.env` file (see
`.env.example`) before connecting Para or Turnkey.
