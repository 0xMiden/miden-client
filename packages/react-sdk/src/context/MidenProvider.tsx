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
import { WebClient } from "@miden-sdk/miden-sdk";
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
  /** Whether the external signer is connected (null = no signer provider) */
  signerConnected: boolean | null;
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
    signerConnected,
    setClient,
    setInitializing,
    setInitError,
    setConfig,
    setSyncState,
    setSignerConnected,
  } = useMidenStore();

  const syncIntervalRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const isInitializedRef = useRef(false);
  const clientLockRef = useRef(new AsyncLock());

  // Track the current signer identity (storeName) to detect identity changes
  const currentStoreNameRef = useRef<string | null>(null);

  // Ref to hold the latest signCb so it can be hot-swapped on the WebClient
  const signCbRef = useRef<
    | ((pubKey: Uint8Array, signingInputs: Uint8Array) => Promise<Uint8Array>)
    | null
  >(null);

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

  // Exposed for advanced consumers who need to serialize custom multi-step
  // operations against the client. Built-in hooks no longer use this since
  // the WebClient handles concurrency internally via Layers 1-3.
  const runExclusive = useCallback(
    async <T,>(fn: () => Promise<T>): Promise<T> =>
      clientLockRef.current.runExclusive(fn),
    []
  );

  // Sync function — the WebClient's Layer 1 (AsyncLock) and Layer 2 (Web
  // Locks) now handle concurrency internally, so we no longer wrap in
  // runExclusive here.
  const sync = useCallback(async () => {
    if (!client || !isReady) return;

    const store = useMidenStore.getState();
    if (store.sync.isSyncing) return;

    setSyncState({ isSyncing: true, error: null });

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
  }, [client, isReady, setSyncState]);

  // Extract stable primitives from signerContext to avoid spurious effect re-runs
  // when the signer provider creates a new object reference on every render.
  const signerIsConnected = signerContext?.isConnected ?? null;
  const signerStoreName = signerContext?.storeName ?? null;
  // Stable identity for accountConfig — only re-init when the account type or
  // storage mode actually changes (publicKeyCommitment is a Uint8Array so we
  // use its length + first byte as a cheap fingerprint; full comparison happens
  // inside initializeSignerAccount).
  const signerAccountType = signerContext?.accountConfig?.accountType ?? null;
  const signerStorageMode =
    signerContext?.accountConfig?.storageMode?.toString() ?? null;

  // Keep signCbRef up to date so the wrapped callback always calls the latest signCb
  useEffect(() => {
    signCbRef.current = signerContext?.signCb ?? null;
  }, [signerContext?.signCb]);

  // Wrapped signCb that reads through the ref — this is passed to WebClient
  // so the callback can be hot-swapped without recreating the client.
  const wrappedSignCb = useCallback(
    async (
      pubKey: Uint8Array,
      signingInputs: Uint8Array
    ): Promise<Uint8Array> => {
      const cb = signCbRef.current;
      if (!cb) {
        throw new Error("Signer is disconnected. Cannot sign.");
      }
      return cb(pubKey, signingInputs);
    },
    []
  );

  // Initialize client
  useEffect(() => {
    // For local keystore mode (no signer), only initialize once
    if (signerIsConnected === null && isInitializedRef.current) return;

    // Signer exists but not connected — mark disconnected, keep client alive
    if (signerIsConnected === false) {
      const store = useMidenStore.getState();

      // Only set signerConnected if it's changing (avoid loops)
      if (store.signerConnected !== false) {
        setSignerConnected(false);
      }

      // Client stays alive — reads continue working, writes are blocked by hooks
      return;
    }

    // Signer is connected — check if we can reuse the existing client
    if (signerIsConnected === true && signerStoreName !== null) {
      const store = useMidenStore.getState();

      // Same identity reconnecting and client already exists — hot-swap signCb
      if (
        currentStoreNameRef.current === signerStoreName &&
        store.client !== null
      ) {
        // Hot-swap the signCb on the existing WebClient instance.
        // The worker reads this.signCb fresh on every callback invocation.
        // signCbRef is already kept in sync by the dedicated useEffect above.
        store.client.setSignCb(wrappedSignCb);
        setSignerConnected(true);
        return;
      }

      // Different identity — clear cached state (but not IndexedDB)
      if (
        currentStoreNameRef.current !== null &&
        currentStoreNameRef.current !== signerStoreName
      ) {
        store.resetInMemoryState();
        // Also clear old client so we get a fresh one for the new identity
        setClient(null);
        setSignerAccountId(null);
      }
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
          let webClient: WebClient;
          let didSignerInit = false;

          if (signerContext && signerIsConnected === true) {
            // External keystore mode - signer provider is present and connected
            const storeName = `MidenClientDB_${signerContext.storeName}`;

            // Update the ref so wrappedSignCb uses the latest callback
            signCbRef.current = signerContext.signCb;

            webClient = await WebClient.createClientWithExternalKeystore(
              resolvedConfig.rpcUrl,
              resolvedConfig.noteTransportUrl,
              resolvedConfig.seed,
              storeName,
              undefined, // getKeyCb - not needed for public accounts
              undefined, // insertKeyCb - not needed for public accounts
              wrappedSignCb
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
            currentStoreNameRef.current = signerContext.storeName;
          } else {
            // No signer provider - standard local keystore (existing behavior)
            const seed = resolvedConfig.seed as Parameters<
              typeof WebClient.createClient
            >[2];
            webClient = await WebClient.createClient(
              resolvedConfig.rpcUrl,
              resolvedConfig.noteTransportUrl,
              seed
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
            if (!signerContext) {
              isInitializedRef.current = true;
            }
            setClient(webClient);
            // Mark signer as connected if in signer mode
            if (signerIsConnected === true) {
              setSignerConnected(true);
            }
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
      // Reset so StrictMode mount-2 can re-init if needed.
      // The cancelled flag prevents mount-1 from setting state after cleanup,
      // and runExclusive queuing ensures mount-2 waits for any in-progress init.
      isInitializedRef.current = false;
    };
  }, [
    runExclusive,
    resolvedConfig,
    setClient,
    setConfig,
    setInitError,
    setInitializing,
    setSyncState,
    setSignerConnected,
    signerIsConnected,
    signerStoreName,
    signerAccountType,
    signerStorageMode,
    wrappedSignCb,
    // Note: signerContext is intentionally NOT a dep — we use stable primitives
    // (signerIsConnected, signerStoreName, signerAccountType, signerStorageMode)
    // to avoid re-running when the signer provider creates a new object ref.
    // signCb changes are handled by the dedicated useEffect + signCbRef above,
    // not by this effect.
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

  // Cross-tab state change listener (Layer 3).
  // The WebClient auto-syncs on cross-tab changes, so the in-memory Rust
  // state is already fresh. We just need to refresh the Zustand store
  // (accounts, sync metadata) so the React UI re-renders.
  // Sync coalescing in the WebClient handles rapid messages without debouncing.
  useEffect(() => {
    if (!isReady || !client) return;

    // The WebClient exposes onStateChanged when BroadcastChannel is available.
    const unsubscribe = client.onStateChanged?.(async () => {
      try {
        const accounts = await client.getAccounts();
        useMidenStore.getState().setAccounts(accounts);
        setSyncState({ lastSyncTime: Date.now() });
      } catch {
        // Non-fatal — next explicit sync will catch up.
      }
    });

    return () => {
      unsubscribe?.();
    };
  }, [isReady, client, setSyncState]);

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
    signerConnected,
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
