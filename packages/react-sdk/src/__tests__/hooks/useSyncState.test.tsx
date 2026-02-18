import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useSyncState } from "../../hooks/useSyncState";
import { useMiden } from "../../context/MidenProvider";
import { useMidenStore } from "../../store/MidenStore";

// Mock useMiden
vi.mock("../../context/MidenProvider", () => ({
  useMiden: vi.fn(),
}));

const mockUseMiden = useMiden as ReturnType<typeof vi.fn>;

beforeEach(() => {
  useMidenStore.getState().reset();
  vi.clearAllMocks();
});

describe("useSyncState", () => {
  describe("initial state", () => {
    it("should return default sync state", () => {
      mockUseMiden.mockReturnValue({
        sync: vi.fn(),
      });

      const { result } = renderHook(() => useSyncState());

      expect(result.current.syncHeight).toBe(0);
      expect(result.current.isSyncing).toBe(false);
      expect(result.current.lastSyncTime).toBeNull();
      expect(result.current.error).toBeNull();
      expect(typeof result.current.sync).toBe("function");
    });
  });

  describe("sync state reflection", () => {
    it("should reflect current sync height from store", () => {
      mockUseMiden.mockReturnValue({
        sync: vi.fn(),
      });

      act(() => {
        useMidenStore.getState().setSyncState({ syncHeight: 150 });
      });

      const { result } = renderHook(() => useSyncState());

      expect(result.current.syncHeight).toBe(150);
    });

    it("should reflect syncing state", () => {
      mockUseMiden.mockReturnValue({
        sync: vi.fn(),
      });

      act(() => {
        useMidenStore.getState().setSyncState({ isSyncing: true });
      });

      const { result } = renderHook(() => useSyncState());

      expect(result.current.isSyncing).toBe(true);
    });

    it("should reflect last sync time", () => {
      mockUseMiden.mockReturnValue({
        sync: vi.fn(),
      });

      const syncTime = Date.now();
      act(() => {
        useMidenStore.getState().setSyncState({ lastSyncTime: syncTime });
      });

      const { result } = renderHook(() => useSyncState());

      expect(result.current.lastSyncTime).toBe(syncTime);
    });

    it("should reflect sync errors", () => {
      mockUseMiden.mockReturnValue({
        sync: vi.fn(),
      });

      const syncError = new Error("Sync failed");
      act(() => {
        useMidenStore.getState().setSyncState({ error: syncError });
      });

      const { result } = renderHook(() => useSyncState());

      expect(result.current.error).toBe(syncError);
    });
  });

  describe("sync function", () => {
    it("should call context sync when invoked", async () => {
      const mockSync = vi.fn().mockResolvedValue(undefined);
      mockUseMiden.mockReturnValue({
        sync: mockSync,
      });

      const { result } = renderHook(() => useSyncState());

      await act(async () => {
        await result.current.sync();
      });

      expect(mockSync).toHaveBeenCalledTimes(1);
    });

    it("should handle sync errors gracefully", async () => {
      const mockSync = vi.fn().mockRejectedValue(new Error("Network error"));
      mockUseMiden.mockReturnValue({
        sync: mockSync,
      });

      const { result } = renderHook(() => useSyncState());

      // Should not throw
      await act(async () => {
        try {
          await result.current.sync();
        } catch {
          // Expected
        }
      });

      expect(mockSync).toHaveBeenCalled();
    });
  });

  describe("reactivity", () => {
    it("should update when sync state changes", () => {
      mockUseMiden.mockReturnValue({
        sync: vi.fn(),
      });

      const { result } = renderHook(() => useSyncState());

      expect(result.current.syncHeight).toBe(0);

      act(() => {
        useMidenStore.getState().setSyncState({ syncHeight: 200 });
      });

      expect(result.current.syncHeight).toBe(200);
    });

    it("should update isSyncing during sync operation", () => {
      mockUseMiden.mockReturnValue({
        sync: vi.fn(),
      });

      const { result } = renderHook(() => useSyncState());

      expect(result.current.isSyncing).toBe(false);

      act(() => {
        useMidenStore.getState().setSyncState({ isSyncing: true });
      });

      expect(result.current.isSyncing).toBe(true);

      act(() => {
        useMidenStore.getState().setSyncState({ isSyncing: false });
      });

      expect(result.current.isSyncing).toBe(false);
    });
  });

  describe("multiple state updates", () => {
    it("should handle multiple rapid state updates", () => {
      mockUseMiden.mockReturnValue({
        sync: vi.fn(),
      });

      const { result } = renderHook(() => useSyncState());

      act(() => {
        useMidenStore.getState().setSyncState({ syncHeight: 100 });
        useMidenStore.getState().setSyncState({ syncHeight: 101 });
        useMidenStore.getState().setSyncState({ syncHeight: 102 });
      });

      expect(result.current.syncHeight).toBe(102);
    });

    it("should preserve other state when updating partial state", () => {
      mockUseMiden.mockReturnValue({
        sync: vi.fn(),
      });

      const syncTime = Date.now();

      act(() => {
        useMidenStore.getState().setSyncState({
          syncHeight: 100,
          lastSyncTime: syncTime,
          isSyncing: false,
          error: null,
        });
      });

      const { result } = renderHook(() => useSyncState());

      // Update only syncHeight
      act(() => {
        useMidenStore.getState().setSyncState({ syncHeight: 150 });
      });

      // Other values should be preserved
      expect(result.current.syncHeight).toBe(150);
      expect(result.current.lastSyncTime).toBe(syncTime);
      expect(result.current.isSyncing).toBe(false);
      expect(result.current.error).toBeNull();
    });
  });

  describe("error recovery", () => {
    it("should clear error on successful sync", () => {
      mockUseMiden.mockReturnValue({
        sync: vi.fn(),
      });

      // Set initial error state
      act(() => {
        useMidenStore.getState().setSyncState({
          error: new Error("Previous error"),
        });
      });

      const { result } = renderHook(() => useSyncState());

      expect(result.current.error).not.toBeNull();

      // Simulate successful sync
      act(() => {
        useMidenStore.getState().setSyncState({
          error: null,
          syncHeight: 200,
          lastSyncTime: Date.now(),
        });
      });

      expect(result.current.error).toBeNull();
      expect(result.current.syncHeight).toBe(200);
    });
  });
});
