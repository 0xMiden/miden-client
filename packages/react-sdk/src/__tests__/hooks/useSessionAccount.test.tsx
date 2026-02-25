import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useSessionAccount } from "../../hooks/useSessionAccount";
import { useMiden } from "../../context/MidenProvider";
import { useMidenStore } from "../../store/MidenStore";
import { createMockWebClient, createMockAccount } from "../mocks/miden-sdk";

vi.mock("../../context/MidenProvider", () => ({
  useMiden: vi.fn(),
}));

const mockUseMiden = useMiden as ReturnType<typeof vi.fn>;

// Mock localStorage
const localStorageMock = (() => {
  let store: Record<string, string> = {};
  return {
    getItem: vi.fn((key: string) => store[key] ?? null),
    setItem: vi.fn((key: string, value: string) => {
      store[key] = value;
    }),
    removeItem: vi.fn((key: string) => {
      delete store[key];
    }),
    key: vi.fn((index: number) => Object.keys(store)[index] ?? null),
    get length() {
      return Object.keys(store).length;
    },
    clear: vi.fn(() => {
      store = {};
    }),
    _reset: () => {
      store = {};
    },
  };
})();

beforeEach(() => {
  useMidenStore.getState().reset();
  localStorageMock._reset();
  vi.stubGlobal("localStorage", localStorageMock);
  vi.clearAllMocks();
});

afterEach(() => {
  vi.unstubAllGlobals();
});

describe("useSessionAccount", () => {
  const defaultOptions = {
    fund: vi.fn().mockResolvedValue(undefined),
    assetId: "0xfaucet",
  };

  describe("initial state", () => {
    it("should return idle state when not initialized", () => {
      mockUseMiden.mockReturnValue({
        client: null,
        isReady: false,
        sync: vi.fn(),
      });

      const { result } = renderHook(() => useSessionAccount(defaultOptions));

      expect(result.current.sessionAccountId).toBeNull();
      expect(result.current.isReady).toBe(false);
      expect(result.current.step).toBe("idle");
      expect(result.current.error).toBeNull();
      expect(typeof result.current.initialize).toBe("function");
      expect(typeof result.current.reset).toBe("function");
    });
  });

  describe("initialize", () => {
    it("should throw when client is not ready", async () => {
      mockUseMiden.mockReturnValue({
        client: null,
        isReady: false,
        sync: vi.fn(),
      });

      const { result } = renderHook(() => useSessionAccount(defaultOptions));

      await expect(result.current.initialize()).rejects.toThrow(
        "Miden client is not ready"
      );
    });

    it("should create wallet and call fund callback", async () => {
      const mockWallet = createMockAccount({
        id: vi.fn(() => ({
          toString: vi.fn(() => "0xsession_wallet"),
          toHex: vi.fn(() => "0xsession_wallet"),
          isFaucet: vi.fn(() => false),
          isRegularAccount: vi.fn(() => true),
          free: vi.fn(),
        })),
      });

      const fundFn = vi.fn().mockResolvedValue(undefined);

      // Mock consumable notes to simulate funding arriving
      const mockConsumableNote = {
        inputNoteRecord: vi.fn(() => ({
          toNote: vi.fn(() => ({})),
        })),
      };

      const mockClient = createMockWebClient({
        newWallet: vi.fn().mockResolvedValue(mockWallet),
        getConsumableNotes: vi
          .fn()
          .mockResolvedValueOnce([]) // First poll: no notes
          .mockResolvedValue([mockConsumableNote]), // Second poll: funding note arrives
        newConsumeTransactionRequest: vi.fn().mockReturnValue({}),
        submitNewTransaction: vi.fn().mockResolvedValue({
          toString: vi.fn(() => "0xtx"),
        }),
      });

      const mockSync = vi.fn().mockResolvedValue(undefined);

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        sync: mockSync,
      });

      const { result } = renderHook(() =>
        useSessionAccount({
          fund: fundFn,
          assetId: "0xfaucet",
          pollIntervalMs: 10, // Fast polling for test
        })
      );

      await act(async () => {
        await result.current.initialize();
      });

      expect(fundFn).toHaveBeenCalledWith("0xsession_wallet");
      expect(result.current.step).toBe("ready");
      expect(result.current.isReady).toBe(true);
      expect(result.current.sessionAccountId).toBe("0xsession_wallet");
    });
  });

  describe("persistence", () => {
    it("should persist session account ID to localStorage", async () => {
      const mockWallet = createMockAccount({
        id: vi.fn(() => ({
          toString: vi.fn(() => "0xpersisted"),
          toHex: vi.fn(() => "0xpersisted"),
          isFaucet: vi.fn(() => false),
          isRegularAccount: vi.fn(() => true),
          free: vi.fn(),
        })),
      });

      const mockConsumableNote = {
        inputNoteRecord: vi.fn(() => ({
          toNote: vi.fn(() => ({})),
        })),
      };

      const mockClient = createMockWebClient({
        newWallet: vi.fn().mockResolvedValue(mockWallet),
        getConsumableNotes: vi.fn().mockResolvedValue([mockConsumableNote]),
        newConsumeTransactionRequest: vi.fn().mockReturnValue({}),
        submitNewTransaction: vi.fn().mockResolvedValue({
          toString: vi.fn(() => "0xtx"),
        }),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        sync: vi.fn(),
      });

      const { result } = renderHook(() =>
        useSessionAccount({
          ...defaultOptions,
          pollIntervalMs: 10,
        })
      );

      await act(async () => {
        await result.current.initialize();
      });

      expect(localStorageMock.setItem).toHaveBeenCalledWith(
        "miden-session:accountId",
        "0xpersisted"
      );
      expect(localStorageMock.setItem).toHaveBeenCalledWith(
        "miden-session:ready",
        "true"
      );
    });

    it("should restore session from localStorage on mount", () => {
      localStorageMock.setItem("miden-session:accountId", "0xrestored");
      localStorageMock.setItem("miden-session:ready", "true");

      mockUseMiden.mockReturnValue({
        client: null,
        isReady: false,
        sync: vi.fn(),
      });

      const { result } = renderHook(() => useSessionAccount(defaultOptions));

      expect(result.current.sessionAccountId).toBe("0xrestored");
      expect(result.current.step).toBe("ready");
      expect(result.current.isReady).toBe(true);
    });

    it("should use custom storage prefix", () => {
      localStorageMock.setItem("custom-prefix:accountId", "0xcustom");
      localStorageMock.setItem("custom-prefix:ready", "true");

      mockUseMiden.mockReturnValue({
        client: null,
        isReady: false,
        sync: vi.fn(),
      });

      const { result } = renderHook(() =>
        useSessionAccount({
          ...defaultOptions,
          storagePrefix: "custom-prefix",
        })
      );

      expect(result.current.sessionAccountId).toBe("0xcustom");
      expect(result.current.isReady).toBe(true);
    });
  });

  describe("reset", () => {
    it("should clear all state and localStorage", () => {
      localStorageMock.setItem("miden-session:accountId", "0xwallet");
      localStorageMock.setItem("miden-session:ready", "true");

      mockUseMiden.mockReturnValue({
        client: null,
        isReady: false,
        sync: vi.fn(),
      });

      const { result } = renderHook(() => useSessionAccount(defaultOptions));

      // Verify restored state
      expect(result.current.sessionAccountId).toBe("0xwallet");

      act(() => {
        result.current.reset();
      });

      expect(result.current.sessionAccountId).toBeNull();
      expect(result.current.step).toBe("idle");
      expect(result.current.isReady).toBe(false);
      expect(result.current.error).toBeNull();
      expect(localStorageMock.removeItem).toHaveBeenCalledWith(
        "miden-session:accountId"
      );
      expect(localStorageMock.removeItem).toHaveBeenCalledWith(
        "miden-session:ready"
      );
    });
  });

  describe("error handling", () => {
    it("should set error when wallet creation fails", async () => {
      const mockClient = createMockWebClient({
        newWallet: vi
          .fn()
          .mockRejectedValue(new Error("Wallet creation failed")),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        sync: vi.fn(),
      });

      const { result } = renderHook(() => useSessionAccount(defaultOptions));

      await act(async () => {
        try {
          await result.current.initialize();
        } catch {
          // Expected
        }
      });

      expect(result.current.error?.message).toBe("Wallet creation failed");
      expect(result.current.step).toBe("idle");
    });
  });
});
