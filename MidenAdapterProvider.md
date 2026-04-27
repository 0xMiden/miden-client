# Unified Signer Interface for MidenProvider

## Goal

Generalize MidenProvider to support multiple wallet/signer implementations (Para, Turnkey, MidenFi) through a unified wrapper provider pattern.

## Key Principles

1. **Single MidenProvider** - no subclasses, no `signer` prop needed
2. **Wrapper provider pattern** - each signer has its own provider that wraps MidenProvider
3. **Context-based detection** - MidenProvider detects signer via `useSignerContext()`
4. **No signer = normal local keystore** - existing behavior unchanged
5. **Reuse existing init code** - signer providers leverage existing functions

## Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│  <ParaSignerProvider apiKey="..." environment="PROD">   │
│    → Creates Para client, manages connection state      │
│    → Provides SignerContext with signCb + accountConfig │
│    ┌─────────────────────────────────────────────────┐  │
│    │  <MidenProvider>                                │  │
│    │    → useSignerContext() detects Para signer     │  │
│    │    → Uses createClientWithExternalKeystore      │  │
│    │    → All existing hooks work unchanged          │  │
│    └─────────────────────────────────────────────────┘  │
│  </ParaSignerProvider>                                  │
└─────────────────────────────────────────────────────────┘
```

## Signer Context Definition

**Location:** `packages/react-sdk/src/context/SignerContext.ts`

```typescript
import { createContext, useContext } from 'react';
import type { AccountType, AccountStorageMode } from '@miden-sdk/miden-sdk';

/**
 * Sign callback for WebClient.createClientWithExternalKeystore
 */
export type SignCallback = (
  pubKey: Uint8Array,
  signingInputs: Uint8Array
) => Promise<Uint8Array>;

/**
 * Account configuration for the signer
 */
export interface SignerAccountConfig {
  /** Public key commitment (for auth component) */
  publicKeyCommitment: Uint8Array;
  /** Account type */
  accountType: AccountType;
  /** Storage mode (public/private/network) */
  storageMode: AccountStorageMode;
  /** Optional seed for deterministic account ID */
  accountSeed?: Uint8Array;
}

/**
 * Context value provided by signer providers
 */
export interface SignerContextValue {
  /** Sign callback for external keystore */
  signCb: SignCallback;
  /** Account config for initialization */
  accountConfig: SignerAccountConfig;
  /** Store name suffix for IndexedDB isolation */
  storeName: string;
  /** Display name for UI (e.g., "Para", "Turnkey", "MidenFi") */
  name: string;
  /** Whether the signer is connected and ready */
  isConnected: boolean;
  /** Connect to the signer (triggers auth flow) */
  connect: () => Promise<void>;
  /** Disconnect from the signer */
  disconnect: () => Promise<void>;
}

/**
 * React context for signer - null when no signer provider is present
 */
export const SignerContext = createContext<SignerContextValue | null>(null);

/**
 * Internal hook for MidenProvider to detect signer context
 */
export function useSignerContext(): SignerContextValue | null {
  return useContext(SignerContext);
}

/**
 * Public hook for apps to interact with the signer
 * Returns null if no signer provider is present (local keystore mode)
 */
export function useSigner(): SignerContextValue | null {
  return useContext(SignerContext);
}
```

## Usage Pattern

All signers use the same wrapper provider pattern - consistent across Para, Turnkey, and MidenFi:

```tsx
// With Para
import { ParaSignerProvider } from '@miden/para';

<ParaSignerProvider apiKey="your-api-key" environment="PRODUCTION">
  <MidenProvider config={{ rpcUrl: "testnet" }}>
    <App />
  </MidenProvider>
</ParaSignerProvider>

// With Turnkey
import { TurnkeySignerProvider } from '@miden/turnkey';

<TurnkeySignerProvider organizationId="your-org-id">
  <MidenProvider config={{ rpcUrl: "testnet" }}>
    <App />
  </MidenProvider>
</TurnkeySignerProvider>

// With MidenFi Wallet
import { MidenFiSignerProvider } from '@demox-labs/miden-wallet-adapter-react';

<MidenFiSignerProvider appName="My App">
  <MidenProvider config={{ rpcUrl: "testnet" }}>
    <App />
  </MidenProvider>
</MidenFiSignerProvider>

// Without signer - standard local keystore (existing behavior)
<MidenProvider config={{ rpcUrl: "testnet" }}>
  <App />
</MidenProvider>
```

### Unified Connection Hook

The `useSigner()` hook works with any signer provider - no need to import signer-specific hooks:

```tsx
import { useSigner } from '@miden-sdk/react';

function ConnectButton() {
  const signer = useSigner();

  // No signer provider = local keystore mode (no connect button needed)
  if (!signer) return null;

  const { isConnected, connect, disconnect, name } = signer;
  return isConnected
    ? <button onClick={disconnect}>Disconnect {name}</button>
    : <button onClick={connect}>Connect with {name}</button>;
}
```

**Switching signers is a one-line change** - just swap the wrapper provider:

```tsx
// Change from Para...
<ParaSignerProvider apiKey="..." environment="PRODUCTION">
  <MidenProvider><App /></MidenProvider>
</ParaSignerProvider>

// ...to Turnkey - ConnectButton code stays identical!
<TurnkeySignerProvider organizationId="...">
  <MidenProvider><App /></MidenProvider>
</TurnkeySignerProvider>
```

### Signer-Specific Hooks (Optional)

For advanced use cases, each provider still exposes its own hook with provider-specific features:

```tsx
// Para-specific: access to Para client, wallet details, etc.
import { useParaSigner } from '@miden/para';
const { para, wallet, ... } = useParaSigner();

// Turnkey-specific: access to Turnkey client, organization, etc.
import { useTurnkeySigner } from '@miden/turnkey';
const { client, organizationId, ... } = useTurnkeySigner();

// MidenFi: uses existing wallet adapter hook
import { useWallet } from '@demox-labs/miden-wallet-adapter-react';
const { wallet, wallets, select, ... } = useWallet();
```

## Implementation Plan

### Phase 1: Create SignerContext (react-sdk)

**Files to create/modify:**

1. **`packages/react-sdk/src/context/SignerContext.ts`** (NEW)
   - Define `SignerContextValue`, `SignCallback`, `SignerAccountConfig` types
   - Create `SignerContext` React context
   - Export `useSignerContext()` hook

2. **`packages/react-sdk/src/types/index.ts`**
   - Export signer types

3. **`packages/react-sdk/src/index.ts`**
   - Export `SignerContext`, `useSigner`, and signer types
   - Note: `useSignerContext` is internal (used by MidenProvider), `useSigner` is the public hook

### Phase 2: Update MidenProvider (react-sdk)

**File:** `packages/react-sdk/src/context/MidenProvider.tsx`

Changes:
- Remove `signer` prop - use context detection instead
- Use `useSignerContext()` to detect if wrapped by a signer provider
- Use `createClientWithExternalKeystore` when signer context is present
- Initialize account using signer's `accountConfig`

```typescript
import { useSignerContext } from './SignerContext';

interface MidenProviderProps {
  children: ReactNode;
  config?: MidenConfig;
  // No signer prop! Detected via context
  loadingComponent?: ReactNode;
  errorComponent?: ReactNode | ((error: Error) => ReactNode);
}

export function MidenProvider({ children, config, ... }) {
  const resolvedConfig = useMemo(...);

  // Detect signer from context (null if no signer provider above)
  const signerContext = useSignerContext();

  useEffect(() => {
    const initClient = async () => {
      let webClient: WebClient;

      if (signerContext && signerContext.isConnected) {
        // External keystore mode - signer provider is present and connected
        const storeName = `MidenClientDB_${signerContext.storeName}`;

        webClient = await WebClient.createClientWithExternalKeystore(
          resolvedConfig.rpcUrl,
          resolvedConfig.noteTransportUrl,
          resolvedConfig.seed,
          storeName,
          undefined, // getKeyCb
          undefined, // insertKeyCb
          signerContext.signCb
        );

        // Initialize account from signer config
        await initializeSignerAccount(webClient, signerContext.accountConfig);
      } else if (!signerContext) {
        // No signer provider - standard local keystore (existing behavior)
        webClient = await WebClient.createClient(
          resolvedConfig.rpcUrl,
          resolvedConfig.noteTransportUrl,
          resolvedConfig.seed
        );
      } else {
        // Signer provider exists but not connected yet - wait for user to connect
        return;
      }

      setClient(webClient);
    };

    initClient();
  }, [signerContext, resolvedConfig, ...]);
}
```

### Phase 3: Add Account Initialization Helper (react-sdk)

**File:** `packages/react-sdk/src/utils/signerAccount.ts` (NEW)

```typescript
export async function initializeSignerAccount(
  client: WebClient,
  config: SignerAccountConfig
): Promise<string> {
  const { AccountBuilder, AccountComponent, AccountStorageMode } =
    await import('@miden-sdk/miden-sdk');

  await client.syncState();

  // Build account with auth component from public key commitment
  const builder = new AccountBuilder(config.accountSeed ?? new Uint8Array(32));
  const account = builder
    .withAuthComponent(
      AccountComponent.createAuthComponentFromCommitment(
        config.publicKeyCommitment,
        1 // ECDSA auth scheme
      )
    )
    .accountType(config.accountType)
    .storageMode(config.storageMode)
    .withBasicWalletComponent()
    .build().account;

  // Import from chain if public/network storage mode
  const isPrivate = config.storageMode.toString() === 'private';
  if (!isPrivate) {
    try {
      await client.importAccountById(account.id());
    } catch {
      // Account doesn't exist on-chain yet, will create locally
    }
  }

  // Ensure account exists locally
  const existing = await client.getAccount(account.id());
  if (!existing) {
    await client.newAccount(account, false);
  }

  await client.syncState();
  return account.id().toString();
}
```

### Phase 4: ParaSignerProvider Implementation (miden-para)

**File:** `~/miden/miden-para/src/ParaSignerProvider.tsx` (NEW)

Provider pattern that wraps children and provides SignerContext. Reuses existing utility functions.

```typescript
import { useState, useEffect, useCallback, useMemo, createContext, useContext } from 'react';
import { ParaWeb, type Wallet, type Environment } from '@getpara/web-sdk';
import { SignerContext, type SignerContextValue } from '@miden-sdk/react';
import { signCb as createSignCb } from './midenClient';  // Reuse existing!
import { evmPkToCommitment, getUncompressedPublicKeyFromWallet } from './utils';

interface ParaSignerProviderProps {
  children: React.ReactNode;
  apiKey: string;
  environment: Environment;
  showSigningModal?: boolean;
}

// Internal context for Para-specific extras (beyond the unified useSigner interface)
interface ParaSignerExtras {
  para: ParaWeb;
  wallet: Wallet | null;
}
const ParaSignerExtrasContext = createContext<ParaSignerExtras | null>(null);

export function ParaSignerProvider({
  children,
  apiKey,
  environment,
  showSigningModal = false,
}: ParaSignerProviderProps) {
  // Create Para client once
  const para = useMemo(() => new ParaWeb({ apiKey, environment }), [apiKey, environment]);

  const [wallet, setWallet] = useState<Wallet | null>(null);
  const [isConnected, setIsConnected] = useState(false);

  // Check connection status
  useEffect(() => {
    let cancelled = false;

    async function checkConnection() {
      try {
        const isLoggedIn = await para.isFullyLoggedIn();
        if (!isLoggedIn || cancelled) {
          setIsConnected(false);
          setWallet(null);
          return;
        }

        const wallets = Object.values(await para.getWallets());
        const evmWallets = wallets.filter((w) => w.type === 'EVM');

        if (evmWallets.length > 0 && !cancelled) {
          setWallet(evmWallets[0]);
          setIsConnected(true);
        }
      } catch {
        if (!cancelled) {
          setIsConnected(false);
          setWallet(null);
        }
      }
    }

    checkConnection();
    const interval = setInterval(checkConnection, 2000);
    return () => { cancelled = true; clearInterval(interval); };
  }, [para]);

  // Connect/disconnect methods (stable references)
  const connect = useCallback(async () => {
    await para.connect();  // Triggers Para login modal
  }, [para]);

  const disconnect = useCallback(async () => {
    await para.logout();
    setIsConnected(false);
    setWallet(null);
  }, [para]);

  // Build signer context (includes connect/disconnect for unified useSigner hook)
  const [signerContext, setSignerContext] = useState<SignerContextValue | null>(null);

  useEffect(() => {
    let cancelled = false;

    async function buildContext() {
      if (!isConnected || !wallet) {
        // Not connected - provide context with connect/disconnect but no signing capability
        setSignerContext({
          signCb: async () => { throw new Error('Not connected'); },
          accountConfig: null as any,  // Not used when not connected
          storeName: '',
          name: 'Para',
          isConnected: false,
          connect,
          disconnect,
        });
        return;
      }

      // Connected - build full context with signing capability
      const publicKey = await getUncompressedPublicKeyFromWallet(para, wallet);
      const commitment = await evmPkToCommitment(publicKey);
      const signCallback = createSignCb(para, wallet, showSigningModal);

      if (!cancelled) {
        const { AccountType, AccountStorageMode } = await import('@miden-sdk/miden-sdk');
        setSignerContext({
          signCb: signCallback,
          accountConfig: {
            publicKeyCommitment: commitment,
            accountType: AccountType.RegularAccountImmutableCode,
            storageMode: AccountStorageMode.public(),
          },
          storeName: `para_${wallet.id}`,
          name: 'Para',
          isConnected: true,
          connect,
          disconnect,
        });
      }
    }

    buildContext();
    return () => { cancelled = true; };
  }, [isConnected, wallet, para, showSigningModal, connect, disconnect]);

  return (
    <ParaSignerExtrasContext.Provider value={{ para, wallet }}>
      <SignerContext.Provider value={signerContext}>
        {children}
      </SignerContext.Provider>
    </ParaSignerExtrasContext.Provider>
  );
}

// Hook for Para-specific extras (access to Para client, wallet details, etc.)
export function useParaSigner(): ParaSignerExtras & { isConnected: boolean } {
  const extras = useContext(ParaSignerExtrasContext);
  const signer = useContext(SignerContext);
  if (!extras) {
    throw new Error('useParaSigner must be used within ParaSignerProvider');
  }
  return { ...extras, isConnected: signer?.isConnected ?? false };
}
```

**Key points:**
- Creates `ParaWeb` client with `useMemo` (stable instance)
- Provides `SignerContext` for MidenProvider to detect
- **Reuses existing `signCb` from `midenClient.ts`**
- Exposes `useParaSigner()` hook for connect/disconnect UI

**Also update:** `~/miden/miden-para/src/index.ts` to export `ParaSignerProvider` and `useParaSigner`

### Phase 5: TurnkeySignerProvider Implementation (miden-turnkey)

**File:** `~/miden/miden-turnkey/src/TurnkeySignerProvider.tsx` (NEW)

Similar provider pattern. Reuses existing `sign` from `midenClient.ts`.

```typescript
import { useState, useEffect, useCallback, useMemo, createContext, useContext } from 'react';
import { TurnkeyBrowserClient } from '@turnkey/sdk-browser';
import { SignerContext, type SignerContextValue } from '@miden-sdk/react';
import { sign } from './midenClient';  // Reuse existing signing logic
import { evmPkToCommitment, fromTurnkeySig } from './utils';

interface TurnkeySignerProviderProps {
  children: React.ReactNode;
  organizationId: string;
  iframeDomain?: string;
}

// Internal context for Turnkey-specific extras
interface TurnkeySignerExtras {
  client: TurnkeyBrowserClient;
  organizationId: string;
  account: any | null;
}
const TurnkeySignerExtrasContext = createContext<TurnkeySignerExtras | null>(null);

export function TurnkeySignerProvider({
  children,
  organizationId,
  iframeDomain = 'https://auth.turnkey.com',
}: TurnkeySignerProviderProps) {
  const client = useMemo(() => new TurnkeyBrowserClient({ iframeDomain }), [iframeDomain]);

  const [account, setAccount] = useState(null);
  const [isConnected, setIsConnected] = useState(false);

  // Check for existing session/wallets
  useEffect(() => {
    // ... similar pattern to Para - check for available wallets
  }, [client]);

  // Connect/disconnect methods (stable references)
  const connect = useCallback(async () => {
    // Trigger Turnkey auth flow
  }, [client]);

  const disconnect = useCallback(async () => {
    setIsConnected(false);
    setAccount(null);
  }, []);

  // Build signer context (includes connect/disconnect for unified useSigner hook)
  const [signerContext, setSignerContext] = useState<SignerContextValue | null>(null);

  useEffect(() => {
    let cancelled = false;

    async function buildContext() {
      if (!isConnected || !account) {
        // Not connected - provide context with connect/disconnect but no signing capability
        setSignerContext({
          signCb: async () => { throw new Error('Not connected'); },
          accountConfig: null as any,
          storeName: '',
          name: 'Turnkey',
          isConnected: false,
          connect,
          disconnect,
        });
        return;
      }

      // Connected - build full context with signing capability
      const commitment = await evmPkToCommitment(account.publicKey);

      const signCb = async (_: Uint8Array, signingInputs: Uint8Array) => {
        const { SigningInputs } = await import('@miden-sdk/miden-sdk');
        const inputs = SigningInputs.deserialize(signingInputs);
        const messageHex = inputs.toCommitment().toHex();

        const sig = await sign(messageHex, { client, organizationId, account });
        return fromTurnkeySig(sig);
      };

      if (!cancelled) {
        const { AccountType, AccountStorageMode } = await import('@miden-sdk/miden-sdk');
        setSignerContext({
          signCb,
          accountConfig: {
            publicKeyCommitment: commitment,
            accountType: AccountType.RegularAccountImmutableCode,
            storageMode: AccountStorageMode.public(),
          },
          storeName: `turnkey_${account.address}`,
          name: 'Turnkey',
          isConnected: true,
          connect,
          disconnect,
        });
      }
    }

    buildContext();
    return () => { cancelled = true; };
  }, [isConnected, account, client, organizationId, connect, disconnect]);

  return (
    <TurnkeySignerExtrasContext.Provider value={{ client, organizationId, account }}>
      <SignerContext.Provider value={signerContext}>
        {children}
      </SignerContext.Provider>
    </TurnkeySignerExtrasContext.Provider>
  );
}

// Hook for Turnkey-specific extras
export function useTurnkeySigner(): TurnkeySignerExtras & { isConnected: boolean } {
  const extras = useContext(TurnkeySignerExtrasContext);
  const signer = useContext(SignerContext);
  if (!extras) throw new Error('useTurnkeySigner must be used within TurnkeySignerProvider');
  return { ...extras, isConnected: signer?.isConnected ?? false };
}
```

**Key points:**
- **Reuses existing `sign` function from `midenClient.ts`**
- Provider pattern consistent with Para
- Exposes `useTurnkeySigner()` hook for connect/disconnect

### Phase 6: MidenFiSignerProvider Implementation (miden-wallet-adapter)

**File:** `~/miden/miden-wallet-adapter/packages/core/react/MidenFiSignerProvider.tsx` (NEW)

Wraps the existing WalletProvider and provides SignerContext. Defaults to MidenWalletAdapter.

```typescript
import { useMemo, useEffect, useState, useCallback, type ReactNode } from 'react';
import { SignerContext, type SignerContextValue } from '@miden-sdk/react';
import { WalletProvider, useWallet } from './';
import { MidenWalletAdapter } from '@demox-labs/miden-wallet-adapter-miden';

interface MidenFiSignerProviderProps {
  children: ReactNode;
  /** App name shown in wallet connection dialog */
  appName?: string;
  autoConnect?: boolean;
}

// Inner component that builds SignerContext from wallet state
function SignerContextBuilder({ children }: { children: ReactNode }) {
  const {
    connected,
    publicKey,
    address,
    signBytes,
    connect: walletConnect,
    disconnect: walletDisconnect,
  } = useWallet();

  // Wrap wallet connect/disconnect to match unified interface
  const connect = useCallback(async () => {
    await walletConnect();
  }, [walletConnect]);

  const disconnect = useCallback(async () => {
    await walletDisconnect();
  }, [walletDisconnect]);

  const [signerContext, setSignerContext] = useState<SignerContextValue | null>(null);

  useEffect(() => {
    async function buildContext() {
      if (!connected || !publicKey || !address || !signBytes) {
        // Not connected - provide context with connect/disconnect but no signing capability
        setSignerContext({
          signCb: async () => { throw new Error('Not connected'); },
          accountConfig: null as any,
          storeName: '',
          name: 'MidenFi',
          isConnected: false,
          connect,
          disconnect,
        });
        return;
      }

      // Connected - build full context with signing capability
      const signCb = async (_: Uint8Array, signingInputs: Uint8Array) => {
        return signBytes(signingInputs, 'signingInputs');
      };

      const { AccountType, AccountStorageMode } = await import('@miden-sdk/miden-sdk');
      setSignerContext({
        signCb,
        accountConfig: {
          publicKeyCommitment: publicKey,
          accountType: AccountType.RegularAccountImmutableCode,
          storageMode: AccountStorageMode.public(),
        },
        storeName: `midenfi_${address}`,
        name: 'MidenFi',
        isConnected: true,
        connect,
        disconnect,
      });
    }

    buildContext();
  }, [connected, publicKey, address, signBytes, connect, disconnect]);

  return (
    <SignerContext.Provider value={signerContext}>
      {children}
    </SignerContext.Provider>
  );
}

export function MidenFiSignerProvider({
  children,
  appName = 'Miden App',
  autoConnect = false,
}: MidenFiSignerProviderProps) {
  // Default to MidenWalletAdapter - currently the only Miden wallet
  const wallets = useMemo(
    () => [new MidenWalletAdapter({ appName })],
    [appName]
  );

  return (
    <WalletProvider wallets={wallets} autoConnect={autoConnect}>
      <SignerContextBuilder>
        {children}
      </SignerContextBuilder>
    </WalletProvider>
  );
}

// Re-export useWallet for advanced use cases (wallet selection, etc.)
export { useWallet } from './';
```

**Key points:**
- Defaults to `MidenWalletAdapter` - no need to import separately
- Just pass `appName` prop (optional, defaults to "Miden App")
- Future-proof: can add `wallets` prop later if other Miden wallets emerge
- Uses `useWallet()` hook for connect/disconnect UI

## Key Design Decisions

### 1. Unified Wrapper Provider Pattern

All signers use the same pattern - wrap MidenProvider with a signer provider:

```tsx
<ParaSignerProvider apiKey="..." environment="PRODUCTION">
  <MidenProvider>
    <App />
  </MidenProvider>
</ParaSignerProvider>
```

This is consistent across Para, Turnkey, and MidenFi.

### 2. Context-based Detection

MidenProvider detects signers via `useSignerContext()`:

```tsx
// Inside MidenProvider.tsx
export function MidenProvider({ config, ... }) {
  // Detect signer from context - null if no signer provider above
  const signerContext = useSignerContext();

  useEffect(() => {
    if (signerContext?.isConnected) {
      // Use createClientWithExternalKeystore
      const client = await WebClient.createClientWithExternalKeystore(
        ...,
        signerContext.signCb
      );
    } else if (!signerContext) {
      // No signer - use standard createClient
    }
    // else: signer exists but not connected - wait for user to connect
  }, [signerContext]);
}
```

No `signer` prop needed - just wrap with the appropriate provider.

### 3. Unified useSigner() Hook

The `SignerContextValue` includes `connect` and `disconnect`, so apps use one hook for all signers:

```tsx
const signer = useSigner();
// signer.isConnected, signer.connect(), signer.disconnect(), signer.name
```

When `signerContext.isConnected` is false, MidenProvider waits for user to connect.
Apps can show a connect button using the same code regardless of signer type.

### 4. Reuse Existing Init Code

Each signer provider **reuses existing functions** from their repos:
- Para: reuses `signCb`, `evmPkToCommitment` from `midenClient.ts`
- Turnkey: reuses `sign`, `fromTurnkeySig` from `midenClient.ts`

This avoids duplicating the complex signing/conversion logic.

### 5. Single MidenProvider - No Subclasses

- No `signer` prop needed
- No signer provider = standard local keystore (existing behavior unchanged)
- All existing hooks (`useSend`, `useAccounts`, etc.) work unchanged regardless of signer

## Files to Modify (Summary)

**In react-sdk (`packages/react-sdk/`):**
- `src/context/SignerContext.ts` - NEW: SignerContext, useSigner() (public), useSignerContext() (internal), types
- `src/utils/signerAccount.ts` - NEW: Account initialization helper
- `src/context/MidenProvider.tsx` - Use useSignerContext() for detection
- `src/types/index.ts` - Export signer types
- `src/index.ts` - Export SignerContext, useSigner, and types

**In miden-para (`~/miden/miden-para/`):**
- `src/ParaSignerProvider.tsx` - NEW: Provider component + useParaSigner hook
- `src/index.ts` - Export `ParaSignerProvider`, `useParaSigner`
- Reuses: `signCb`, `evmPkToCommitment`, `fromHexSig` from existing files

**In miden-turnkey (`~/miden/miden-turnkey/`):**
- `src/TurnkeySignerProvider.tsx` - NEW: Provider component + useTurnkeySigner hook
- `src/index.ts` - Export `TurnkeySignerProvider`, `useTurnkeySigner`
- Reuses: `sign`, `evmPkToCommitment`, `fromTurnkeySig` from existing files

**In miden-wallet-adapter (`~/miden/miden-wallet-adapter/`):**
- `packages/core/react/MidenFiSignerProvider.tsx` - NEW: Wraps WalletProvider + provides SignerContext
- `packages/core/react/index.ts` - Export `MidenFiSignerProvider`

## Verification Plan

1. **Unit tests** - Mock signer contexts and verify client creation
2. **Integration test with Para** - Use Para testnet credentials
3. **Integration test with Turnkey** - Use Turnkey sandbox
4. **Integration test with MidenFi** - Use wallet extension on testnet
5. **Verify existing hooks** - `useAccounts`, `useSend`, etc. work unchanged

## Design Decisions (Confirmed)

1. **Interface location**: SignerContext + types defined in `@miden-sdk/react` - signers import from there
2. **Unified wrapper pattern**: All signers use `<XxxSignerProvider>` wrapper, no `signer` prop needed
3. **Unified connect UX**: `useSigner()` hook works with any signer - switching providers requires zero app code changes
4. **Signer-specific hooks**: Optional `useParaSigner()`, `useTurnkeySigner()`, `useWallet()` for advanced use cases
5. **Account handling**: Import + create - try import from chain first, create locally if not found
6. **Reuse existing code**: Signer providers import existing functions from their repos, no duplication

## Dependencies Pulled at Import Time

When you `import { ParaSignerProvider } from '@miden/para'`:
- Para SDK deps (`@getpara/web-sdk`) are bundled with that package
- No Para dependency in react-sdk itself
- Tree-shaking works correctly - unused signers aren't bundled

## Remaining Considerations

1. Package naming: Keep signers in their existing repos (`@miden/para`, `@miden/turnkey`, `@demox-labs/miden-wallet-adapter-react`)
2. Runtime signer switching requires remounting the provider (acceptable limitation)
