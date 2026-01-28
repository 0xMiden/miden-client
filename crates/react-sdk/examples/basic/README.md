# Miden React SDK Example

A minimal example demonstrating how to use `@miden-sdk/react` hooks in a React application.

## Features Demonstrated

- **MidenProvider** - Initializing the Miden client with WASM
- **useMiden** - Accessing client ready state
- **useSyncState** - Syncing with the Miden network
- **useAccounts** - Listing wallets and faucets
- **useCreateWallet** - Creating new wallets
- **useCreateFaucet** - Creating new token faucets
- **useMint** - Minting tokens from a faucet
- **useSend** - Sending tokens between accounts

## Running the Example

1. First, build the React SDK:

```bash
cd ../..
yarn install
yarn build
```

2. Install example dependencies:

```bash
yarn install
```

3. Start the development server:

```bash
yarn dev
```

4. Open http://localhost:5173 in your browser

## Configuration

By default, the example connects to the Miden testnet. You can change the RPC URL in `src/main.tsx`:

```tsx
<MidenProvider
  config={{
    rpcUrl: "https://rpc.testnet.miden.io",
  }}
>
```

## Project Structure

```
examples/basic/
├── src/
│   ├── main.tsx           # Entry point with MidenProvider
│   ├── App.tsx            # Main app component
│   └── components/
│       ├── WalletManager.tsx   # Wallet creation UI
│       ├── FaucetManager.tsx   # Faucet creation UI
│       └── TransferForm.tsx    # Mint & send tokens UI
├── index.html
├── package.json
└── vite.config.ts
```

## Usage Patterns

### Basic Setup

```tsx
import { MidenProvider, useMiden } from "@miden-sdk/react";

function App() {
  return (
    <MidenProvider>
      <MyComponent />
    </MidenProvider>
  );
}

function MyComponent() {
  const { isReady, error } = useMiden();

  if (!isReady) return <div>Loading...</div>;
  if (error) return <div>Error: {error.message}</div>;

  return <div>Connected to Miden!</div>;
}
```

### Creating a Wallet

```tsx
import { useCreateWallet } from "@miden-sdk/react";

function CreateWalletButton() {
  const { createWallet, isCreating } = useCreateWallet();

  const handleClick = async () => {
    const wallet = await createWallet({ storageMode: "private" });
    console.log("Created:", wallet.id().toString());
  };

  return (
    <button onClick={handleClick} disabled={isCreating}>
      {isCreating ? "Creating..." : "Create Wallet"}
    </button>
  );
}
```

### Sending Tokens

```tsx
import { useSend } from "@miden-sdk/react";

function SendTokens() {
  const { send, isLoading, stage } = useSend();

  const handleSend = async () => {
    await send({
      from: "0x...",
      to: "0x...",
      assetId: "0x...",
      amount: 100n,
    });
  };

  return (
    <button onClick={handleSend} disabled={isLoading}>
      {isLoading ? `Sending (${stage})...` : "Send"}
    </button>
  );
}
```
