import { describe, it, expect, beforeEach } from "vitest";
import { useMidenStore } from "../../store/MidenStore";
import {
  createMockWebClient,
  createMockAccountHeader,
  createMockInputNoteRecord,
  createMockConsumableNoteRecord,
  createMockAccount,
} from "../mocks/miden-sdk";

beforeEach(() => {
  useMidenStore.getState().reset();
});

describe("MidenStore", () => {
  describe("initial state", () => {
    it("should have correct initial state", () => {
      const state = useMidenStore.getState();

      expect(state.client).toBeNull();
      expect(state.isReady).toBe(false);
      expect(state.isInitializing).toBe(false);
      expect(state.initError).toBeNull();
      expect(state.config).toEqual({});

      expect(state.sync.syncHeight).toBe(0);
      expect(state.sync.isSyncing).toBe(false);
      expect(state.sync.lastSyncTime).toBeNull();
      expect(state.sync.error).toBeNull();

      expect(state.accounts).toEqual([]);
      expect(state.accountDetails.size).toBe(0);
      expect(state.notes).toEqual([]);
      expect(state.consumableNotes).toEqual([]);

      expect(state.isLoadingAccounts).toBe(false);
      expect(state.isLoadingNotes).toBe(false);
    });
  });

  describe("setClient", () => {
    it("should set client and mark as ready", () => {
      const mockClient = createMockWebClient();

      useMidenStore.getState().setClient(mockClient as any);

      const state = useMidenStore.getState();
      expect(state.client).toBe(mockClient);
      expect(state.isReady).toBe(true);
      expect(state.isInitializing).toBe(false);
      expect(state.initError).toBeNull();
    });
  });

  describe("setInitializing", () => {
    it("should set initializing state", () => {
      useMidenStore.getState().setInitializing(true);
      expect(useMidenStore.getState().isInitializing).toBe(true);

      useMidenStore.getState().setInitializing(false);
      expect(useMidenStore.getState().isInitializing).toBe(false);
    });
  });

  describe("setInitError", () => {
    it("should set error and mark as not initializing/ready", () => {
      const error = new Error("Init failed");

      useMidenStore.getState().setInitError(error);

      const state = useMidenStore.getState();
      expect(state.initError).toBe(error);
      expect(state.isInitializing).toBe(false);
      expect(state.isReady).toBe(false);
    });

    it("should clear error when set to null", () => {
      useMidenStore.getState().setInitError(new Error("Error"));
      useMidenStore.getState().setInitError(null);

      expect(useMidenStore.getState().initError).toBeNull();
    });
  });

  describe("setConfig", () => {
    it("should set config", () => {
      const config = {
        rpcUrl: "https://test.rpc",
        autoSyncInterval: 30000,
      };

      useMidenStore.getState().setConfig(config);

      expect(useMidenStore.getState().config).toEqual(config);
    });
  });

  describe("setSyncState", () => {
    it("should update sync state partially", () => {
      useMidenStore.getState().setSyncState({ syncHeight: 100 });
      expect(useMidenStore.getState().sync.syncHeight).toBe(100);
      expect(useMidenStore.getState().sync.isSyncing).toBe(false);

      useMidenStore.getState().setSyncState({ isSyncing: true });
      expect(useMidenStore.getState().sync.syncHeight).toBe(100);
      expect(useMidenStore.getState().sync.isSyncing).toBe(true);
    });

    it("should update multiple sync properties", () => {
      const syncTime = Date.now();
      useMidenStore.getState().setSyncState({
        syncHeight: 200,
        isSyncing: false,
        lastSyncTime: syncTime,
        error: null,
      });

      const sync = useMidenStore.getState().sync;
      expect(sync.syncHeight).toBe(200);
      expect(sync.isSyncing).toBe(false);
      expect(sync.lastSyncTime).toBe(syncTime);
      expect(sync.error).toBeNull();
    });

    it("should set sync error", () => {
      const syncError = new Error("Sync failed");
      useMidenStore.getState().setSyncState({ error: syncError });

      expect(useMidenStore.getState().sync.error).toBe(syncError);
    });
  });

  describe("setAccounts", () => {
    it("should set accounts", () => {
      const accounts = [
        createMockAccountHeader("0x1"),
        createMockAccountHeader("0x2"),
      ];

      useMidenStore.getState().setAccounts(accounts as any);

      expect(useMidenStore.getState().accounts).toEqual(accounts);
    });

    it("should replace existing accounts", () => {
      const accounts1 = [createMockAccountHeader("0x1")];
      const accounts2 = [
        createMockAccountHeader("0x2"),
        createMockAccountHeader("0x3"),
      ];

      useMidenStore.getState().setAccounts(accounts1 as any);
      expect(useMidenStore.getState().accounts.length).toBe(1);

      useMidenStore.getState().setAccounts(accounts2 as any);
      expect(useMidenStore.getState().accounts.length).toBe(2);
    });
  });

  describe("setAccountDetails", () => {
    it("should add account details to map", () => {
      const account = createMockAccount();

      useMidenStore.getState().setAccountDetails("0x123", account as any);

      const details = useMidenStore.getState().accountDetails;
      expect(details.size).toBe(1);
      expect(details.get("0x123")).toBe(account);
    });

    it("should update existing account details", () => {
      const account1 = createMockAccount();
      const account2 = createMockAccount();

      useMidenStore.getState().setAccountDetails("0x123", account1 as any);
      useMidenStore.getState().setAccountDetails("0x123", account2 as any);

      const details = useMidenStore.getState().accountDetails;
      expect(details.size).toBe(1);
      expect(details.get("0x123")).toBe(account2);
    });

    it("should handle multiple accounts", () => {
      const account1 = createMockAccount();
      const account2 = createMockAccount();

      useMidenStore.getState().setAccountDetails("0x1", account1 as any);
      useMidenStore.getState().setAccountDetails("0x2", account2 as any);

      const details = useMidenStore.getState().accountDetails;
      expect(details.size).toBe(2);
    });
  });

  describe("setNotes", () => {
    it("should set notes", () => {
      const notes = [
        createMockInputNoteRecord("0xnote1"),
        createMockInputNoteRecord("0xnote2"),
      ];

      useMidenStore.getState().setNotes(notes as any);

      expect(useMidenStore.getState().notes).toEqual(notes);
    });
  });

  describe("setConsumableNotes", () => {
    it("should set consumable notes", () => {
      const notes = [
        createMockConsumableNoteRecord("0xnote1"),
        createMockConsumableNoteRecord("0xnote2"),
      ];

      useMidenStore.getState().setConsumableNotes(notes as any);

      expect(useMidenStore.getState().consumableNotes).toEqual(notes);
    });
  });

  describe("setLoadingAccounts", () => {
    it("should set loading accounts state", () => {
      useMidenStore.getState().setLoadingAccounts(true);
      expect(useMidenStore.getState().isLoadingAccounts).toBe(true);

      useMidenStore.getState().setLoadingAccounts(false);
      expect(useMidenStore.getState().isLoadingAccounts).toBe(false);
    });
  });

  describe("setLoadingNotes", () => {
    it("should set loading notes state", () => {
      useMidenStore.getState().setLoadingNotes(true);
      expect(useMidenStore.getState().isLoadingNotes).toBe(true);

      useMidenStore.getState().setLoadingNotes(false);
      expect(useMidenStore.getState().isLoadingNotes).toBe(false);
    });
  });

  describe("reset", () => {
    it("should reset all state to initial values", () => {
      // Set some state
      useMidenStore.getState().setClient(createMockWebClient() as any);
      useMidenStore.getState().setAccounts([createMockAccountHeader()] as any);
      useMidenStore.getState().setSyncState({ syncHeight: 100 });
      useMidenStore.getState().setLoadingAccounts(true);

      // Reset
      useMidenStore.getState().reset();

      // Verify all reset
      const state = useMidenStore.getState();
      expect(state.client).toBeNull();
      expect(state.isReady).toBe(false);
      expect(state.accounts).toEqual([]);
      expect(state.sync.syncHeight).toBe(0);
      expect(state.isLoadingAccounts).toBe(false);
    });

    it("should clear account details map", () => {
      useMidenStore
        .getState()
        .setAccountDetails("0x1", createMockAccount() as any);
      useMidenStore
        .getState()
        .setAccountDetails("0x2", createMockAccount() as any);

      expect(useMidenStore.getState().accountDetails.size).toBe(2);

      useMidenStore.getState().reset();

      expect(useMidenStore.getState().accountDetails.size).toBe(0);
    });
  });

  describe("selector hooks", () => {
    it("should provide useClient selector", () => {
      const mockClient = createMockWebClient();
      useMidenStore.getState().setClient(mockClient as any);

      // Test via getState (since we can't use React hooks directly in tests)
      expect(useMidenStore.getState().client).toBe(mockClient);
    });

    it("should provide useIsReady selector", () => {
      expect(useMidenStore.getState().isReady).toBe(false);

      useMidenStore.getState().setClient(createMockWebClient() as any);

      expect(useMidenStore.getState().isReady).toBe(true);
    });

    it("should provide useSyncStateStore selector", () => {
      useMidenStore.getState().setSyncState({ syncHeight: 50 });

      expect(useMidenStore.getState().sync.syncHeight).toBe(50);
    });
  });

  describe("state persistence", () => {
    it("should maintain state between getState calls", () => {
      useMidenStore.getState().setSyncState({ syncHeight: 100 });
      useMidenStore.getState().setAccounts([createMockAccountHeader()] as any);

      // Multiple getState calls should return same state
      expect(useMidenStore.getState().sync.syncHeight).toBe(100);
      expect(useMidenStore.getState().accounts.length).toBe(1);
    });

    it("should allow sequential updates", () => {
      const store = useMidenStore.getState();

      store.setSyncState({ syncHeight: 1 });
      store.setSyncState({ syncHeight: 2 });
      store.setSyncState({ syncHeight: 3 });

      expect(useMidenStore.getState().sync.syncHeight).toBe(3);
    });
  });

  describe("noteFirstSeen tracking", () => {
    it("should record firstSeen timestamp when setNotes is called", () => {
      const note1 = createMockInputNoteRecord("0xnote1");
      const note2 = createMockInputNoteRecord("0xnote2");

      const before = Date.now();
      useMidenStore.getState().setNotes([note1, note2] as any);
      const after = Date.now();

      const firstSeen = useMidenStore.getState().noteFirstSeen;
      expect(firstSeen.has("0xnote1")).toBe(true);
      expect(firstSeen.has("0xnote2")).toBe(true);
      expect(firstSeen.get("0xnote1")!).toBeGreaterThanOrEqual(before);
      expect(firstSeen.get("0xnote1")!).toBeLessThanOrEqual(after);
    });

    it("should not overwrite existing firstSeen timestamps", () => {
      const note = createMockInputNoteRecord("0xnote1");

      useMidenStore.getState().setNotes([note] as any);
      const first = useMidenStore.getState().noteFirstSeen.get("0xnote1")!;

      // Wait a tick and set again
      useMidenStore.getState().setNotes([note] as any);
      const second = useMidenStore.getState().noteFirstSeen.get("0xnote1")!;

      expect(second).toBe(first); // Should be the same timestamp
    });

    it("should be empty in initial state", () => {
      expect(useMidenStore.getState().noteFirstSeen.size).toBe(0);
    });

    it("should prune stale entries when notes are removed via setNotes", () => {
      const note1 = createMockInputNoteRecord("0xnote1");
      const note2 = createMockInputNoteRecord("0xnote2");

      useMidenStore.getState().setNotes([note1, note2] as any);
      expect(useMidenStore.getState().noteFirstSeen.size).toBe(2);

      // Remove note2 from the list
      useMidenStore.getState().setNotes([note1] as any);
      const firstSeen = useMidenStore.getState().noteFirstSeen;
      expect(firstSeen.size).toBe(1);
      expect(firstSeen.has("0xnote1")).toBe(true);
      expect(firstSeen.has("0xnote2")).toBe(false);
    });

    it("should prune stale entries when notes are removed via setNotesIfChanged", () => {
      const note1 = createMockInputNoteRecord("0xnote1");
      const note2 = createMockInputNoteRecord("0xnote2");

      useMidenStore.getState().setNotes([note1, note2] as any);
      expect(useMidenStore.getState().noteFirstSeen.size).toBe(2);

      // Remove note2
      useMidenStore.getState().setNotesIfChanged([note1] as any);
      const firstSeen = useMidenStore.getState().noteFirstSeen;
      expect(firstSeen.size).toBe(1);
      expect(firstSeen.has("0xnote1")).toBe(true);
      expect(firstSeen.has("0xnote2")).toBe(false);
    });
  });

  describe("setNotesIfChanged", () => {
    it("should update notes when IDs change", () => {
      const note1 = createMockInputNoteRecord("0xnote1");
      const note2 = createMockInputNoteRecord("0xnote2");

      useMidenStore.getState().setNotes([note1] as any);
      useMidenStore.getState().setNotesIfChanged([note1, note2] as any);

      expect(useMidenStore.getState().notes.length).toBe(2);
    });

    it("should skip update when note IDs are the same", () => {
      const note1 = createMockInputNoteRecord("0xnote1");
      const note2 = createMockInputNoteRecord("0xnote2");

      useMidenStore.getState().setNotes([note1, note2] as any);
      const firstRef = useMidenStore.getState().notes;

      // Same IDs, possibly different objects
      const note1b = createMockInputNoteRecord("0xnote1");
      const note2b = createMockInputNoteRecord("0xnote2");
      useMidenStore.getState().setNotesIfChanged([note1b, note2b] as any);

      // Should be the same reference (no update triggered)
      expect(useMidenStore.getState().notes).toBe(firstRef);
    });

    it("should track firstSeen for new notes", () => {
      const note1 = createMockInputNoteRecord("0xnote1");
      useMidenStore.getState().setNotes([note1] as any);

      const note2 = createMockInputNoteRecord("0xnote2");
      useMidenStore.getState().setNotesIfChanged([note1, note2] as any);

      expect(useMidenStore.getState().noteFirstSeen.has("0xnote2")).toBe(true);
    });
  });

  describe("setConsumableNotesIfChanged", () => {
    it("should update consumable notes when IDs change", () => {
      const cn1 = createMockConsumableNoteRecord("0xcn1");
      const cn2 = createMockConsumableNoteRecord("0xcn2");

      useMidenStore.getState().setConsumableNotes([cn1] as any);
      useMidenStore.getState().setConsumableNotesIfChanged([cn1, cn2] as any);

      expect(useMidenStore.getState().consumableNotes.length).toBe(2);
    });

    it("should skip update when consumable note IDs are the same", () => {
      const cn1 = createMockConsumableNoteRecord("0xcn1");
      useMidenStore.getState().setConsumableNotes([cn1] as any);
      const firstRef = useMidenStore.getState().consumableNotes;

      const cn1b = createMockConsumableNoteRecord("0xcn1");
      useMidenStore.getState().setConsumableNotesIfChanged([cn1b] as any);

      expect(useMidenStore.getState().consumableNotes).toBe(firstRef);
    });
  });
});
