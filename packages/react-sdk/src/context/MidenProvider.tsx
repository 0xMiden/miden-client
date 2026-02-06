import {
  createContext,
  useContext,
  useEffect,
  useRef,
  useCallback,
  useMemo,
  type ReactNode,
} from "react";
import { WebClient } from "@miden-sdk/miden-sdk";
import { useMidenStore } from "../store/MidenStore";
import type { MidenConfig } from "../types";
import { DEFAULTS } from "../types";
import { AsyncLock } from "../utils/asyncLock";
import { resolveRpcUrl } from "../utils/network";
import { resolveTransactionProver } from "../utils/prover";

interface MidenContextValue {
  client: WebClient | null;
  isReady: boolean;
  isInitializing: boolean;
  error: Error | null;
  sync: () => Promise<void>;
  runExclusive: <T>(fn: () => Promise<T>) => Promise<T>;
  prover: ReturnType<typeof resolveTransactionProver>;
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
    if (isInitializedRef.current) return;
    isInitializedRef.current = true;

    const initClient = async () => {
      setInitializing(true);
      setConfig(resolvedConfig);

      try {
        // SDK factory methods now take a single config object (not positional args).
        const webClient = await WebClient.createClient({
          rpcUrl: resolvedConfig.rpcUrl,
          noteTransportUrl: resolvedConfig.noteTransportUrl,
          seed: resolvedConfig.seed,
        });

        setClient(webClient);

        // Initial sync
        try {
          const summary = await runExclusive(() => webClient.syncState());
          setSyncState({
            syncHeight: summary.blockNum(),
            lastSyncTime: Date.now(),
          });
        } catch {
          // Initial sync failure is non-fatal
        }
      } catch (error) {
        setInitError(error instanceof Error ? error : new Error(String(error)));
      }
    };

    initClient();
  }, [
    runExclusive,
    resolvedConfig,
    setClient,
    setConfig,
    setInitError,
    setInitializing,
    setSyncState,
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
