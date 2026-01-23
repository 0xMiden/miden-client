import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, act, waitFor } from "@testing-library/react";
import { useAccount } from "../../hooks/useAccount";
import { useMiden } from "../../context/MidenProvider";
import { useMidenStore } from "../../store/MidenStore";
import {
  createMockWebClient,
  createMockAccount,
  createMockAccountId,
  createMockVault,
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

describe("useAccount", () => {
  describe("initial state", () => {
    it("should return null account when no id provided", () => {
      mockUseMiden.mockReturnValue({
        client: null,
        isReady: false,
      });

      const { result } = renderHook(() => useAccount(undefined));

      expect(result.current.account).toBeNull();
      expect(result.current.assets).toEqual([]);
      expect(result.current.isLoading).toBe(false);
      expect(result.current.error).toBeNull();
    });

    it("should return null account when client is not ready", () => {
      mockUseMiden.mockReturnValue({
        client: null,
        isReady: false,
      });

      const { result } = renderHook(() => useAccount("0x1234"));

      expect(result.current.account).toBeNull();
      expect(result.current.isLoading).toBe(false);
    });
  });

  describe("fetching account", () => {
    it("should fetch account by string ID", async () => {
      const accountId = "0x1234567890abcdef";
      const mockAccount = createMockAccount();
      mockAccount.id = vi.fn(() => createMockAccountId(accountId));

      const mockClient = createMockWebClient({
        getAccount: vi.fn().mockResolvedValue(mockAccount),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
      });

      act(() => {
        useMidenStore.getState().setClient(mockClient as any);
      });

      const { result } = renderHook(() => useAccount(accountId));

      await waitFor(() => {
        expect(result.current.account).not.toBeNull();
      });

      expect(mockClient.getAccount).toHaveBeenCalled();
    });

    it("should fetch account by AccountId object", async () => {
      const accountIdObj = createMockAccountId("0xabcdef1234567890");
      const mockAccount = createMockAccount();

      const mockClient = createMockWebClient({
        getAccount: vi.fn().mockResolvedValue(mockAccount),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
      });

      act(() => {
        useMidenStore.getState().setClient(mockClient as any);
      });

      const { result } = renderHook(() => useAccount(accountIdObj as any));

      await waitFor(() => {
        expect(result.current.account).not.toBeNull();
      });
    });

    it("should cache fetched accounts", async () => {
      const accountId = "0x1234567890abcdef";
      const mockAccount = createMockAccount();

      const mockClient = createMockWebClient({
        getAccount: vi.fn().mockResolvedValue(mockAccount),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
      });

      act(() => {
        useMidenStore.getState().setClient(mockClient as any);
      });

      // First render
      const { result, rerender } = renderHook(() => useAccount(accountId));

      await waitFor(() => {
        expect(result.current.account).not.toBeNull();
      });

      // Should have called once
      expect(mockClient.getAccount).toHaveBeenCalledTimes(1);

      // Rerender - should use cached value
      rerender();

      // Should still be only one call
      expect(mockClient.getAccount).toHaveBeenCalledTimes(1);
    });

    it("should handle account not found", async () => {
      const mockClient = createMockWebClient({
        getAccount: vi.fn().mockResolvedValue(null),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
      });

      act(() => {
        useMidenStore.getState().setClient(mockClient as any);
      });

      const { result } = renderHook(() => useAccount("0xnonexistent"));

      await waitFor(() => {
        expect(result.current.isLoading).toBe(false);
      });

      expect(result.current.account).toBeNull();
    });
  });

  describe("assets extraction", () => {
    it("should extract assets from account vault", async () => {
      const accountId = "0x1234567890abcdef";
      const mockAssets = [
        { faucetId: "0xfaucet1", amount: 1000n },
        { faucetId: "0xfaucet2", amount: 500n },
      ];

      const mockAccount = createMockAccount();
      mockAccount.vault = vi.fn(() => createMockVault(mockAssets));

      const mockClient = createMockWebClient({
        getAccount: vi.fn().mockResolvedValue(mockAccount),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
      });

      act(() => {
        useMidenStore.getState().setClient(mockClient as any);
        useMidenStore.getState().setAccountDetails(accountId, mockAccount as any);
      });

      const { result } = renderHook(() => useAccount(accountId));

      expect(result.current.assets.length).toBe(2);
      expect(result.current.assets[0].faucetId).toBeDefined();
      expect(result.current.assets[0].amount).toBeDefined();
    });

    it("should return empty assets when vault throws", async () => {
      const accountId = "0x1234567890abcdef";
      const mockAccount = createMockAccount();
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      (mockAccount as any).vault = vi.fn(() => {
        throw new Error("Vault error");
      });

      const mockClient = createMockWebClient();

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
      });

      act(() => {
        useMidenStore.getState().setClient(mockClient as any);
        useMidenStore.getState().setAccountDetails(accountId, mockAccount as any);
      });

      const { result } = renderHook(() => useAccount(accountId));

      expect(result.current.assets).toEqual([]);
    });
  });

  describe("getBalance helper", () => {
    it("should return balance for specific faucet", async () => {
      const accountId = "0x1234567890abcdef";
      const faucetId = "0xfaucet1";
      const mockAssets = [{ faucetId, amount: 1000n }];

      const mockAccount = createMockAccount();
      mockAccount.vault = vi.fn(() => createMockVault(mockAssets));

      const mockClient = createMockWebClient();

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
      });

      act(() => {
        useMidenStore.getState().setClient(mockClient as any);
        useMidenStore.getState().setAccountDetails(accountId, mockAccount as any);
      });

      const { result } = renderHook(() => useAccount(accountId));

      const balance = result.current.getBalance(faucetId);
      expect(balance).toBe(1000n);
    });

    it("should return 0n for unknown faucet", async () => {
      const accountId = "0x1234567890abcdef";

      const mockClient = createMockWebClient();

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
      });

      act(() => {
        useMidenStore.getState().setClient(mockClient as any);
        useMidenStore.getState().setAccountDetails(accountId, createMockAccount() as any);
      });

      const { result } = renderHook(() => useAccount(accountId));

      const balance = result.current.getBalance("0xunknown");
      expect(balance).toBe(0n);
    });
  });

  describe("refetch", () => {
    it("should refetch account data", async () => {
      const accountId = "0x1234567890abcdef";
      const mockAccount1 = createMockAccount();
      const mockAccount2 = createMockAccount();
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      (mockAccount2 as any).nonce = vi.fn(() => ({ toString: () => "2" }));

      const mockClient = createMockWebClient({
        getAccount: vi
          .fn()
          .mockResolvedValueOnce(mockAccount1)
          .mockResolvedValueOnce(mockAccount2),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
      });

      act(() => {
        useMidenStore.getState().setClient(mockClient as any);
      });

      const { result } = renderHook(() => useAccount(accountId));

      // Wait for initial fetch
      await waitFor(() => {
        expect(result.current.account).not.toBeNull();
      });

      // Refetch
      await act(async () => {
        await result.current.refetch();
      });

      expect(mockClient.getAccount).toHaveBeenCalledTimes(2);
    });
  });

  describe("error handling", () => {
    it("should capture fetch errors", async () => {
      const accountId = "0x1234567890abcdef";
      const fetchError = new Error("Network error");

      const mockClient = createMockWebClient({
        getAccount: vi.fn().mockRejectedValue(fetchError),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
      });

      act(() => {
        useMidenStore.getState().setClient(mockClient as any);
      });

      const { result } = renderHook(() => useAccount(accountId));

      await waitFor(() => {
        expect(result.current.error).not.toBeNull();
      });

      expect(result.current.error?.message).toBe("Network error");
      expect(result.current.isLoading).toBe(false);
    });
  });

  describe("loading state", () => {
    it("should track loading state during fetch", async () => {
      const accountId = "0x1234567890abcdef";
      let resolvePromise: (value: any) => void;
      const fetchPromise = new Promise((resolve) => {
        resolvePromise = resolve;
      });

      const mockClient = createMockWebClient({
        getAccount: vi.fn().mockReturnValue(fetchPromise),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
      });

      act(() => {
        useMidenStore.getState().setClient(mockClient as any);
      });

      const { result } = renderHook(() => useAccount(accountId));

      // Trigger fetch
      act(() => {
        result.current.refetch();
      });

      // Should be loading
      await waitFor(() => {
        expect(result.current.isLoading).toBe(true);
      });

      // Resolve
      await act(async () => {
        resolvePromise!(createMockAccount());
      });

      await waitFor(() => {
        expect(result.current.isLoading).toBe(false);
      });
    });
  });
});
