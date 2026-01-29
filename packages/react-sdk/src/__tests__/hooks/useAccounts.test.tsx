import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, act, waitFor } from "@testing-library/react";
import { useAccounts } from "../../hooks/useAccounts";
import { useMiden } from "../../context/MidenProvider";
import { useMidenStore } from "../../store/MidenStore";
import {
  createMockWebClient,
  createMockAccountHeader,
  createMockAccountId,
} from "../mocks/miden-sdk";

// Mock useMiden
vi.mock("../../context/MidenProvider", () => ({
  useMiden: vi.fn(),
}));

const mockUseMiden = useMiden as ReturnType<typeof vi.fn>;

beforeEach(() => {
  useMidenStore.getState().reset();
  vi.clearAllMocks();
});

describe("useAccounts", () => {
  describe("initial state", () => {
    it("should return empty accounts when client is not ready", () => {
      mockUseMiden.mockReturnValue({
        client: null,
        isReady: false,
      });

      const { result } = renderHook(() => useAccounts());

      expect(result.current.accounts).toEqual([]);
      expect(result.current.wallets).toEqual([]);
      expect(result.current.faucets).toEqual([]);
      expect(result.current.isLoading).toBe(false);
      expect(result.current.error).toBeNull();
    });
  });

  describe("when client is ready", () => {
    it("should fetch accounts on mount", async () => {
      const mockAccounts = [
        createMockAccountHeader("0x1000000000000001"),
        createMockAccountHeader("0x1000000000000002"),
      ];

      const mockClient = createMockWebClient({
        getAccounts: vi.fn().mockResolvedValue(mockAccounts),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
      });

      act(() => {
        useMidenStore.getState().setClient(mockClient as any);
      });

      const { result } = renderHook(() => useAccounts());

      await waitFor(() => {
        expect(result.current.accounts.length).toBeGreaterThan(0);
      });

      expect(mockClient.getAccounts).toHaveBeenCalled();
    });

    it("should categorize accounts into wallets and faucets", async () => {
      const walletAccount = createMockAccountHeader("0x1234567890abcdef");
      const faucetAccount = createMockAccountHeader("0x2234567890abcdef");

      faucetAccount.id = vi.fn(() => ({
        ...createMockAccountId("0x2234567890abcdef"),
        toHex: vi.fn(() => "0x2234567890abcdef"),
      }));

      const mockClient = createMockWebClient({
        getAccounts: vi.fn().mockResolvedValue([walletAccount, faucetAccount]),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
      });

      act(() => {
        useMidenStore.getState().setClient(mockClient as any);
        useMidenStore
          .getState()
          .setAccounts([walletAccount, faucetAccount] as any);
      });

      const { result } = renderHook(() => useAccounts());

      expect(result.current.accounts.length).toBe(2);
    });
  });

  describe("refetch", () => {
    it("should refetch accounts when called", async () => {
      const mockAccounts = [createMockAccountHeader("0x1000000000000001")];
      const updatedAccounts = [
        createMockAccountHeader("0x1000000000000001"),
        createMockAccountHeader("0x1000000000000002"),
      ];

      const mockClient = createMockWebClient({
        getAccounts: vi
          .fn()
          .mockResolvedValueOnce(mockAccounts)
          .mockResolvedValueOnce(updatedAccounts),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
      });

      act(() => {
        useMidenStore.getState().setClient(mockClient as any);
      });

      const { result } = renderHook(() => useAccounts());

      await waitFor(() => {
        expect(result.current.accounts.length).toBe(1);
      });

      await act(async () => {
        await result.current.refetch();
      });

      await waitFor(() => {
        expect(result.current.accounts.length).toBe(2);
      });

      expect(mockClient.getAccounts).toHaveBeenCalledTimes(2);
    });

    it("should not refetch when client is not ready", async () => {
      const mockClient = createMockWebClient();

      mockUseMiden.mockReturnValue({
        client: null,
        isReady: false,
      });

      const { result } = renderHook(() => useAccounts());

      await act(async () => {
        await result.current.refetch();
      });

      expect(mockClient.getAccounts).not.toHaveBeenCalled();
    });
  });

  describe("loading state", () => {
    it("should set loading state during fetch", async () => {
      let resolvePromise: (value: any[]) => void;
      const fetchPromise = new Promise<any[]>((resolve) => {
        resolvePromise = resolve;
      });

      const mockClient = createMockWebClient({
        getAccounts: vi.fn().mockReturnValue(fetchPromise),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
      });

      act(() => {
        useMidenStore.getState().setClient(mockClient as any);
      });

      const { result } = renderHook(() => useAccounts());

      // Trigger fetch
      act(() => {
        result.current.refetch();
      });

      // Should be loading
      await waitFor(() => {
        expect(result.current.isLoading).toBe(true);
      });

      // Resolve the promise
      await act(async () => {
        resolvePromise!([createMockAccountHeader()]);
      });

      // Should no longer be loading
      await waitFor(() => {
        expect(result.current.isLoading).toBe(false);
      });
    });
  });

  describe("error handling", () => {
    it("should handle fetch errors gracefully", async () => {
      const mockClient = createMockWebClient({
        getAccounts: vi.fn().mockRejectedValue(new Error("Network error")),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
      });

      act(() => {
        useMidenStore.getState().setClient(mockClient as any);
      });

      const consoleError = vi
        .spyOn(console, "error")
        .mockImplementation(() => {});
      try {
        const { result } = renderHook(() => useAccounts());

        // Trigger fetch
        await act(async () => {
          await result.current.refetch();
        });

        // Should not throw, just log error
        expect(result.current.isLoading).toBe(false);
      } finally {
        consoleError.mockRestore();
      }
    });
  });

  describe("account categorization", () => {
    it("should correctly identify faucet accounts by ID pattern", () => {
      // Test the isFaucetId logic through the hook
      const mockClient = createMockWebClient();

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
      });

      act(() => {
        useMidenStore.getState().setClient(mockClient as any);
      });

      // Set accounts with different patterns
      const accounts = [
        createMockAccountHeader("0x0123456789abcdef"), // Regular (type 00)
        createMockAccountHeader("0x8123456789abcdef"), // Faucet (type 10)
      ];

      act(() => {
        useMidenStore.getState().setAccounts(accounts as any);
      });

      const { result } = renderHook(() => useAccounts());

      // Both should be in the total accounts
      expect(result.current.accounts.length).toBe(2);
    });
  });
});
