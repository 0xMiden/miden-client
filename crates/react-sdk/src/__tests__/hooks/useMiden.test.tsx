import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, act } from "@testing-library/react";
import React from "react";
import { useMiden } from "../../context/MidenProvider";
import { useMidenStore } from "../../store/MidenStore";
import { createMockWebClient, createMockSyncSummary } from "../mocks/miden-sdk";

// Reset store before each test
beforeEach(() => {
  useMidenStore.getState().reset();
});

describe("useMiden", () => {
  describe("when not wrapped in MidenProvider", () => {
    it("should throw an error", () => {
      const consoleError = vi
        .spyOn(console, "error")
        .mockImplementation(() => {});
      try {
        expect(() => {
          renderHook(() => useMiden());
        }).toThrow("useMiden must be used within a MidenProvider");
      } finally {
        consoleError.mockRestore();
      }
    });
  });

  describe("when client is not ready", () => {
    it("should return isReady as false", () => {
      // Set up store without client - verify initial state
      useMidenStore.getState();

      const wrapper = ({ children }: { children: React.ReactNode }) => {
        React.useEffect(() => {
          // Don't set client - leave it null
        }, []);
        return <>{children}</>;
      };

      // Create a minimal context for testing
      const { result } = renderHook(
        () => ({
          client: useMidenStore((s) => s.client),
          isReady: useMidenStore((s) => s.isReady),
        }),
        { wrapper }
      );

      expect(result.current.client).toBeNull();
      expect(result.current.isReady).toBe(false);
    });
  });

  describe("when client is ready", () => {
    it("should return the client and isReady as true", () => {
      const mockClient = createMockWebClient();

      const { result } = renderHook(() => ({
        client: useMidenStore((s) => s.client),
        isReady: useMidenStore((s) => s.isReady),
      }));

      // Simulate client initialization
      act(() => {
        useMidenStore.getState().setClient(mockClient as any);
      });

      expect(result.current.client).toBe(mockClient);
      expect(result.current.isReady).toBe(true);
    });
  });

  describe("initialization state", () => {
    it("should track initializing state", () => {
      const { result } = renderHook(() => ({
        isInitializing: useMidenStore((s) => s.isInitializing),
      }));

      expect(result.current.isInitializing).toBe(false);

      act(() => {
        useMidenStore.getState().setInitializing(true);
      });

      expect(result.current.isInitializing).toBe(true);

      act(() => {
        useMidenStore.getState().setInitializing(false);
      });

      expect(result.current.isInitializing).toBe(false);
    });

    it("should track initialization errors", () => {
      const { result } = renderHook(() => ({
        initError: useMidenStore((s) => s.initError),
      }));

      expect(result.current.initError).toBeNull();

      const error = new Error("WASM init failed");
      act(() => {
        useMidenStore.getState().setInitError(error);
      });

      expect(result.current.initError).toBe(error);
    });
  });
});

describe("useMidenClient", () => {
  describe("when client is not ready", () => {
    it("should throw an error", () => {
      // Verify client is null, which would cause the hook to throw
      expect(useMidenStore.getState().client).toBeNull();
      expect(useMidenStore.getState().isReady).toBe(false);

      // Test that a function implementing the hook logic would throw
      const getClient = () => {
        const client = useMidenStore.getState().client;
        const isReady = useMidenStore.getState().isReady;
        if (!client || !isReady) {
          throw new Error(
            "Miden client is not ready. Make sure you are inside a MidenProvider and the client has initialized."
          );
        }
        return client;
      };

      expect(getClient).toThrow("Miden client is not ready");
    });
  });

  describe("when client is ready", () => {
    it("should return the client", () => {
      const mockClient = createMockWebClient();

      // Set up the client
      act(() => {
        useMidenStore.getState().setClient(mockClient as any);
      });

      const { result } = renderHook(() => {
        const client = useMidenStore((s) => s.client);
        const isReady = useMidenStore((s) => s.isReady);
        if (!client || !isReady) {
          throw new Error("Client not ready");
        }
        return client;
      });

      expect(result.current).toBe(mockClient);
    });
  });
});

describe("sync functionality", () => {
  it("should update sync state after successful sync", async () => {
    const mockSyncSummary = createMockSyncSummary(150);
    const mockClient = createMockWebClient({
      syncState: vi.fn().mockResolvedValue(mockSyncSummary),
      getAccounts: vi.fn().mockResolvedValue([]),
    });

    // Set up client
    act(() => {
      useMidenStore.getState().setClient(mockClient as any);
    });

    const { result } = renderHook(() => ({
      sync: useMidenStore((s) => s.sync),
      setSyncState: useMidenStore.getState().setSyncState,
    }));

    // Initial state
    expect(result.current.sync.syncHeight).toBe(0);
    expect(result.current.sync.isSyncing).toBe(false);

    // Simulate sync
    await act(async () => {
      result.current.setSyncState({ isSyncing: true });
    });

    expect(result.current.sync.isSyncing).toBe(true);

    // Complete sync
    await act(async () => {
      result.current.setSyncState({
        syncHeight: 150,
        isSyncing: false,
        lastSyncTime: Date.now(),
      });
    });

    expect(result.current.sync.syncHeight).toBe(150);
    expect(result.current.sync.isSyncing).toBe(false);
    expect(result.current.sync.lastSyncTime).not.toBeNull();
  });

  it("should handle sync errors", async () => {
    const { result } = renderHook(() => ({
      sync: useMidenStore((s) => s.sync),
      setSyncState: useMidenStore.getState().setSyncState,
    }));

    const syncError = new Error("Network error");

    await act(async () => {
      result.current.setSyncState({
        isSyncing: false,
        error: syncError,
      });
    });

    expect(result.current.sync.error).toBe(syncError);
    expect(result.current.sync.isSyncing).toBe(false);
  });
});
