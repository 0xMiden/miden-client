import {
  createContext,
  useContext,
  useEffect,
  useRef,
  useCallback,
  type ReactNode,
} from "react";
import { WebClient } from "@miden-sdk/miden-sdk";
import { useMidenStore } from "../store/MidenStore";
import type { MidenConfig } from "../types";
import { DEFAULTS } from "../types";

interface MidenContextValue {
  client: WebClient | null;
  isReady: boolean;
  isInitializing: boolean;
  error: Error | null;
  sync: () => Promise<void>;
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

  // Sync function
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

  // Initialize client
  useEffect(() => {
    if (isInitializedRef.current) return;
    isInitializedRef.current = true;

    const initClient = async () => {
      setInitializing(true);
      setConfig(config);

      try {
        const webClient = new WebClient();
        await webClient.createClient(
          config.rpcUrl,
          config.noteTransportUrl,
          config.seed
        );

        setClient(webClient);

        // Initial sync
        try {
          const summary = await webClient.syncState();
          setSyncState({
            syncHeight: summary.blockNum(),
            lastSyncTime: Date.now(),
          });
        } catch {
          // Initial sync failure is non-fatal
        }
      } catch (error) {
        setInitError(
          error instanceof Error ? error : new Error(String(error))
        );
      }
    };

    initClient();
  }, [config, setClient, setConfig, setInitError, setInitializing, setSyncState]);

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

  // Render loading state
  if (isInitializing) {
    if (loadingComponent) {
      return <>{loadingComponent}</>;
    }
    return null;
  }

  // Render error state
  if (initError) {
    if (errorComponent) {
      if (typeof errorComponent === "function") {
        return <>{errorComponent(initError)}</>;
      }
      return <>{errorComponent}</>;
    }
    return null;
  }

  const contextValue: MidenContextValue = {
    client,
    isReady,
    isInitializing,
    error: initError,
    sync,
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
