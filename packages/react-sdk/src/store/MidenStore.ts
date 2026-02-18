import { create } from "zustand";
import type {
  WebClient,
  Account,
  AccountHeader,
  InputNoteRecord,
  ConsumableNoteRecord,
} from "@miden-sdk/miden-sdk";
import type { SyncState, MidenConfig, AssetMetadata } from "../types";

interface MidenStoreState {
  // Client state
  client: WebClient | null;
  isReady: boolean;
  isInitializing: boolean;
  initError: Error | null;
  config: MidenConfig;

  // Sync state
  sync: SyncState;

  // Cached data
  accounts: AccountHeader[];
  accountDetails: Map<string, Account>;
  notes: InputNoteRecord[];
  consumableNotes: ConsumableNoteRecord[];
  assetMetadata: Map<string, AssetMetadata>;

  // Loading states
  isLoadingAccounts: boolean;
  isLoadingNotes: boolean;

  // Actions
  setClient: (client: WebClient | null) => void;
  setInitializing: (isInitializing: boolean) => void;
  setInitError: (error: Error | null) => void;
  setConfig: (config: MidenConfig) => void;

  setSyncState: (sync: Partial<SyncState>) => void;

  setAccounts: (accounts: AccountHeader[]) => void;
  setAccountDetails: (accountId: string, account: Account) => void;
  setNotes: (notes: InputNoteRecord[]) => void;
  setConsumableNotes: (notes: ConsumableNoteRecord[]) => void;
  setAssetMetadata: (assetId: string, metadata: AssetMetadata) => void;

  setLoadingAccounts: (isLoading: boolean) => void;
  setLoadingNotes: (isLoading: boolean) => void;

  reset: () => void;
}

const initialSyncState: SyncState = {
  syncHeight: 0,
  isSyncing: false,
  lastSyncTime: null,
  error: null,
};

const initialState = {
  client: null,
  isReady: false,
  isInitializing: false,
  initError: null,
  config: {},

  sync: initialSyncState,

  accounts: [],
  accountDetails: new Map<string, Account>(),
  notes: [],
  consumableNotes: [],
  assetMetadata: new Map<string, AssetMetadata>(),

  isLoadingAccounts: false,
  isLoadingNotes: false,
};

export const useMidenStore = create<MidenStoreState>()((set) => ({
  ...initialState,

  setClient: (client) =>
    set({
      client,
      isReady: client !== null,
      isInitializing: false,
      initError: null,
    }),

  setInitializing: (isInitializing) => set({ isInitializing }),

  setInitError: (initError) =>
    set({
      initError,
      isInitializing: false,
      isReady: false,
    }),

  setConfig: (config) => set({ config }),

  setSyncState: (sync) =>
    set((state) => ({
      sync: { ...state.sync, ...sync },
    })),

  setAccounts: (accounts) => set({ accounts }),

  setAccountDetails: (accountId, account) =>
    set((state) => {
      const newMap = new Map(state.accountDetails);
      newMap.set(accountId, account);
      return { accountDetails: newMap };
    }),

  setNotes: (notes) => set({ notes }),

  setConsumableNotes: (consumableNotes) => set({ consumableNotes }),

  setAssetMetadata: (assetId, metadata) =>
    set((state) => {
      const newMap = new Map(state.assetMetadata);
      newMap.set(assetId, metadata);
      return { assetMetadata: newMap };
    }),

  setLoadingAccounts: (isLoadingAccounts) => set({ isLoadingAccounts }),

  setLoadingNotes: (isLoadingNotes) => set({ isLoadingNotes }),

  reset: () => set(initialState),
}));

// Selector hooks for optimal re-renders
export const useClient = () => useMidenStore((state) => state.client);
export const useIsReady = () => useMidenStore((state) => state.isReady);
export const useIsInitializing = () =>
  useMidenStore((state) => state.isInitializing);
export const useInitError = () => useMidenStore((state) => state.initError);
export const useConfig = () => useMidenStore((state) => state.config);
export const useSyncStateStore = () => useMidenStore((state) => state.sync);
export const useAccountsStore = () => useMidenStore((state) => state.accounts);
export const useNotesStore = () => useMidenStore((state) => state.notes);
export const useConsumableNotesStore = () =>
  useMidenStore((state) => state.consumableNotes);
export const useAssetMetadataStore = () =>
  useMidenStore((state) => state.assetMetadata);
