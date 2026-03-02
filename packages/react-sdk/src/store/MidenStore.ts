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

  // Signer connection state (null = no signer provider)
  signerConnected: boolean | null;

  // Sync state
  sync: SyncState;

  // Cached data
  accounts: AccountHeader[];
  accountDetails: Map<string, Account>;
  notes: InputNoteRecord[];
  consumableNotes: ConsumableNoteRecord[];
  assetMetadata: Map<string, AssetMetadata>;

  // Temporal note tracking — records when each note ID was first observed
  noteFirstSeen: Map<string, number>;

  // Loading states
  isLoadingAccounts: boolean;
  isLoadingNotes: boolean;

  // Actions
  setClient: (client: WebClient | null) => void;
  setInitializing: (isInitializing: boolean) => void;
  setInitError: (error: Error | null) => void;
  setConfig: (config: MidenConfig) => void;
  setSignerConnected: (connected: boolean | null) => void;

  setSyncState: (sync: Partial<SyncState>) => void;

  setAccounts: (accounts: AccountHeader[]) => void;
  setAccountDetails: (accountId: string, account: Account) => void;
  setNotes: (notes: InputNoteRecord[]) => void;
  setNotesIfChanged: (notes: InputNoteRecord[]) => void;
  setConsumableNotes: (notes: ConsumableNoteRecord[]) => void;
  setConsumableNotesIfChanged: (notes: ConsumableNoteRecord[]) => void;
  setAssetMetadata: (assetId: string, metadata: AssetMetadata) => void;

  setLoadingAccounts: (isLoading: boolean) => void;
  setLoadingNotes: (isLoading: boolean) => void;

  /** Clear cached data (accounts, notes, metadata, sync) but keep client, isReady, and config */
  resetInMemoryState: () => void;
  reset: () => void;
}

const initialSyncState: SyncState = {
  syncHeight: 0,
  isSyncing: false,
  lastSyncTime: null,
  error: null,
};

function freshCachedState() {
  return {
    sync: { ...initialSyncState },

    accounts: [] as AccountHeader[],
    accountDetails: new Map<string, Account>(),
    notes: [] as InputNoteRecord[],
    consumableNotes: [] as ConsumableNoteRecord[],
    assetMetadata: new Map<string, AssetMetadata>(),
    noteFirstSeen: new Map<string, number>(),

    isLoadingAccounts: false,
    isLoadingNotes: false,
  };
}

function freshState() {
  return {
    client: null as WebClient | null,
    isReady: false,
    isInitializing: false,
    initError: null as Error | null,
    config: {} as MidenConfig,
    signerConnected: null as boolean | null,

    ...freshCachedState(),
  };
}

export const useMidenStore = create<MidenStoreState>()((set) => ({
  ...freshState(),

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

  setSignerConnected: (signerConnected) => set({ signerConnected }),

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

  setNotes: (notes) =>
    set((state) => {
      const now = Date.now();
      const newFirstSeen = new Map<string, number>();
      for (const note of notes) {
        try {
          const id = note.id().toString();
          newFirstSeen.set(id, state.noteFirstSeen.get(id) ?? now);
        } catch {
          // Skip if id() fails
        }
      }
      return { notes, noteFirstSeen: newFirstSeen };
    }),

  setNotesIfChanged: (notes) =>
    set((state) => {
      const safeId = (n: InputNoteRecord): string | null => {
        try {
          return n.id().toString();
        } catch {
          return null;
        }
      };
      const prevIds = new Set<string>();
      for (const n of state.notes) {
        const id = safeId(n);
        if (id) prevIds.add(id);
      }
      const newIds = new Set<string>();
      for (const n of notes) {
        const id = safeId(n);
        if (id) newIds.add(id);
      }
      if (
        prevIds.size === newIds.size &&
        [...prevIds].every((id) => newIds.has(id))
      ) {
        return {};
      }
      const now = Date.now();
      const newFirstSeen = new Map<string, number>();
      for (const note of notes) {
        try {
          const id = note.id().toString();
          // Preserve existing timestamp or record new one
          newFirstSeen.set(id, state.noteFirstSeen.get(id) ?? now);
        } catch {
          // Skip
        }
      }
      return { notes, noteFirstSeen: newFirstSeen };
    }),

  setConsumableNotes: (consumableNotes) => set({ consumableNotes }),

  setConsumableNotesIfChanged: (consumableNotes) =>
    set((state) => {
      const safeId = (n: ConsumableNoteRecord): string | null => {
        try {
          return n.inputNoteRecord().id().toString();
        } catch {
          return null;
        }
      };
      const prevIds = new Set<string>();
      for (const n of state.consumableNotes) {
        const id = safeId(n);
        if (id) prevIds.add(id);
      }
      const newIds = new Set<string>();
      for (const n of consumableNotes) {
        const id = safeId(n);
        if (id) newIds.add(id);
      }
      if (
        prevIds.size === newIds.size &&
        [...prevIds].every((id) => newIds.has(id))
      ) {
        return {};
      }
      return { consumableNotes };
    }),

  setAssetMetadata: (assetId, metadata) =>
    set((state) => {
      const newMap = new Map(state.assetMetadata);
      newMap.set(assetId, metadata);
      return { assetMetadata: newMap };
    }),

  setLoadingAccounts: (isLoadingAccounts) => set({ isLoadingAccounts }),

  setLoadingNotes: (isLoadingNotes) => set({ isLoadingNotes }),

  resetInMemoryState: () => set(freshCachedState()),

  reset: () => set(freshState()),
}));

// Selector hooks for optimal re-renders
export const useClient = () => useMidenStore((state) => state.client);
export const useIsReady = () => useMidenStore((state) => state.isReady);
export const useSignerConnected = () =>
  useMidenStore((state) => state.signerConnected);
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
export const useNoteFirstSeenStore = () =>
  useMidenStore((state) => state.noteFirstSeen);
