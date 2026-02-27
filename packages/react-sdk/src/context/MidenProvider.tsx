import {
  createContext,
  useContext,
  useEffect,
  useRef,
  useCallback,
  useMemo,
  useState,
  type ReactNode,
} from "react";
import { WasmWebClient as WebClient } from "@miden-sdk/miden-sdk";
import { useMidenStore } from "../store/MidenStore";
import type { MidenConfig } from "../types";
import { DEFAULTS } from "../types";
import { AsyncLock } from "../utils/asyncLock";
import { resolveRpcUrl } from "../utils/network";
import { resolveTransactionProver } from "../utils/prover";
import { useSigner } from "./SignerContext";
import { initializeSignerAccount } from "../utils/signerAccount";

interface MidenContextValue {
  client: WebClient | null;
  isReady: boolean;
  isInitializing: boolean;
  error: Error | null;
  sync: () => Promise<void>;
  runExclusive: <T>(fn: () => Promise<T>) => Promise<T>;
  prover: ReturnType<typeof resolveTransactionProver>;
  /** Account ID from signer (only set when using external signer) */
  signerAccountId: string | null;
}

const MidenContext = createContext<MidenContextValue | null>(null);

interface MidenProviderProps {
  children: ReactNode;
  config?: MidenConfig;
  /** Custom loading component shown during WASM initialization */
  loadingComponent?: ReactNode;
  /** Custom error component shown if initialization fails */
  errorComponent?: ReactNode | ((error: Error) => ReactNode);
}

export function MidenProvider({
  children,
  config = {},
  loadingComponent,
  errorComponent,
}: MidenProviderProps) {
  const {
    client,
    isReady,
    isInitializing,
    initError,
    setClient,
    setInitializing,
    setInitError,
    setConfig,
    setSyncState,
  } = useMidenStore();

  const syncIntervalRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const isInitializedRef = useRef(false);
  const clientLockRef = useRef(new AsyncLock());

  // Detect signer from context (null if no signer provider above)
  const signerContext = useSigner();
  const [signerAccountId, setSignerAccountId] = useState<string | null>(null);

  const resolvedConfig = useMemo(
    () => ({
      ...config,
      rpcUrl: resolveRpcUrl(config.rpcUrl),
    }),
    [config]
  );
  const defaultProver = useMemo(
    () => resolveTransactionProver(resolvedConfig),
    [
      resolvedConfig.prover,
      resolvedConfig.proverTimeoutMs,
      resolvedConfig.proverUrls?.devnet,
      resolvedConfig.proverUrls?.testnet,
    ]
  );

  const runExclusive = useCallback(
    async <T,>(fn: () => Promise<T>): Promise<T> =>
      clientLockRef.current.runExclusive(fn),
    []
  );

  // Sync function
  const sync = useCallback(async () => {
    if (!client || !isReady) return;

    const store = useMidenStore.getState();
    if (store.sync.isSyncing) return;

    setSyncState({ isSyncing: true, error: null });

    await runExclusive(async () => {
      try {
        const summary = await client.syncState();
        const syncHeight = summary.blockNum();

        setSyncState({
          syncHeight,
          isSyncing: false,
          lastSyncTime: Date.now(),
          error: null,
        });

        // Trigger account and note refresh after sync
        const accounts = await client.getAccounts();
        useMidenStore.getState().setAccounts(accounts);
      } catch (error) {
        setSyncState({
          isSyncing: false,
          error: error instanceof Error ? error : new Error(String(error)),
        });
      }
    });
  }, [client, isReady, runExclusive, setSyncState]);

  // Initialize client
  useEffect(() => {
    // For signer mode, we need to re-initialize when connection state changes
    // For local keystore mode, we only initialize once
    if (!signerContext && isInitializedRef.current) return;

    // If signer provider exists but not connected, wait for user to connect
    if (signerContext && !signerContext.isConnected) {
      // Reset state when signer disconnects
      // Read client from store directly (not closure) since client is not in deps
      if (useMidenStore.getState().client) {
        useMidenStore.getState().reset();
        setClient(null);
        setSignerAccountId(null);
      }
      return;
    }

    // Mark as initialized for local keystore mode
    if (!signerContext) {
      isInitializedRef.current = true;
    }

    let cancelled = false;

    // Wrap the entire init in runExclusive so that if the effect re-triggers
    // while a previous init is still running, the new init waits for the old
    // one to finish.  This prevents concurrent WASM access (which crashes
    // with "recursive use of an object detected").
    const initClient = async () => {
      await runExclusive(async () => {
        // Re-check cancelled after potentially waiting for the lock
        if (cancelled) return;

        setInitializing(true);
        setConfig(resolvedConfig);

        try {
          let webClient: WebClient | undefined;
          let didSignerInit = false;

          if (signerContext && signerContext.isConnected) {
            // External keystore mode - signer provider is present and connected
            const storeName = `MidenClientDB_${signerContext.storeName}`;

            webClient = await WebClient.createClientWithExternalKeystore(
              resolvedConfig.rpcUrl,
              resolvedConfig.noteTransportUrl,
              resolvedConfig.seed,
              storeName,
              undefined, // getKeyCb - not needed for public accounts
              undefined, // insertKeyCb - not needed for public accounts
              signerContext.signCb
            );

            if (cancelled) return;

            // Initialize account from signer config
            // (this already syncs the client internally)
            const accountId = await initializeSignerAccount(
              webClient,
              signerContext.accountConfig
            );
            if (cancelled) return;
            setSignerAccountId(accountId);
            didSignerInit = true;
          } else if (resolvedConfig.passkeyEncryption) {
            // Passkey encryption mode — local keystore with encrypted keys.
            // Fall back to standard (unencrypted) mode if the browser doesn't
            // support WebAuthn PRF (e.g. Firefox, older browsers).
            const { createPasskeyKeystore, isPasskeyPrfSupported } =
              await import("@miden-sdk/miden-sdk");
            const supported = await isPasskeyPrfSupported();

            if (supported) {
              const passkeyOpts =
                typeof resolvedConfig.passkeyEncryption === "object"
                  ? resolvedConfig.passkeyEncryption
                  : {};
              const storeName = resolvedConfig.storeName || "default";
              const keystore = await createPasskeyKeystore(
                storeName,
                passkeyOpts
              );
              if (cancelled) return;

              webClient = await WebClient.createClientWithExternalKeystore(
                resolvedConfig.rpcUrl,
                resolvedConfig.noteTransportUrl,
                resolvedConfig.seed,
                storeName,
                keystore.getKey,
                keystore.insertKey,
                undefined // sign — Rust signs locally using getKey
              );
              if (cancelled) return;
            }
            // else: fall through to standard createClient below
          }

          if (!webClient) {
            // Standard local keystore (no signer, no passkey or unsupported)
            const seed = resolvedConfig.seed as Parameters<
              typeof WebClient.createClient
            >[2];
            webClient = await WebClient.createClient(
              resolvedConfig.rpcUrl,
              resolvedConfig.noteTransportUrl,
              seed,
              resolvedConfig.storeName
            );
            if (cancelled) return;
          }

          // Initial sync BEFORE setClient — setClient atomically sets isReady=true
          // which triggers auto-sync and consumer hooks. Doing sync first avoids
          // concurrent WASM access between init sync and auto-sync.
          // Skip for signer mode: initializeSignerAccount already synced.
          if (!didSignerInit) {
            try {
              const summary = await webClient.syncState();
              if (cancelled) return;
              setSyncState({
                syncHeight: summary.blockNum(),
                lastSyncTime: Date.now(),
              });
            } catch {
              // Initial sync failure is non-fatal
            }
          }

          // Load accounts before making client ready
          if (!cancelled) {
            try {
              const accounts = await webClient.getAccounts();
              if (cancelled) return;
              useMidenStore.getState().setAccounts(accounts);
            } catch {
              // Non-fatal
            }
          }

          // Set client LAST — this atomically sets isReady=true and
          // isInitializing=false, which enables auto-sync and consumer hooks.
          if (!cancelled) {
            setClient(webClient);
          }
        } catch (error) {
          if (!cancelled) {
            setInitError(
              error instanceof Error ? error : new Error(String(error))
            );
          }
        }
      });
    };

    initClient();
    return () => {
      cancelled = true;
      // Reset so StrictMode's second invocation can re-trigger init
      if (!signerContext) {
        isInitializedRef.current = false;
      }
    };
  }, [
    runExclusive,
    resolvedConfig,
    setClient,
    setConfig,
    setInitError,
    setInitializing,
    setSyncState,
    signerContext,
  ]);

  // Auto-sync interval
  useEffect(() => {
    if (!isReady || !client) return;

    const interval = config.autoSyncInterval ?? DEFAULTS.AUTO_SYNC_INTERVAL;
    if (interval <= 0) return;

    syncIntervalRef.current = setInterval(() => {
      sync();
    }, interval);

    return () => {
      if (syncIntervalRef.current) {
        clearInterval(syncIntervalRef.current);
        syncIntervalRef.current = null;
      }
    };
  }, [isReady, client, config.autoSyncInterval, sync]);

  // Render loading state when a custom component is provided.
  if (isInitializing && loadingComponent) {
    return <>{loadingComponent}</>;
  }

  // Render error state when a custom component is provided.
  if (initError && errorComponent) {
    if (typeof errorComponent === "function") {
      return <>{errorComponent(initError)}</>;
    }
    return <>{errorComponent}</>;
  }

  const contextValue: MidenContextValue = {
    client,
    isReady,
    isInitializing,
    error: initError,
    sync,
    runExclusive,
    prover: defaultProver,
    signerAccountId,
  };

  return (
    <MidenContext.Provider value={contextValue}>
      {children}
    </MidenContext.Provider>
  );
}

export function useMiden(): MidenContextValue {
  const context = useContext(MidenContext);
  if (!context) {
    throw new Error("useMiden must be used within a MidenProvider");
  }
  return context;
}

export function useMidenClient(): WebClient {
  const { client, isReady } = useMiden();
  if (!client || !isReady) {
    throw new Error(
      "Miden client is not ready. Make sure you are inside a MidenProvider and the client has initialized."
    );
  }
  return client;
}
