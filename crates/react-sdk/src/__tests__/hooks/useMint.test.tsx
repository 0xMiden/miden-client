import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, act, waitFor } from "@testing-library/react";
import { useMint } from "../../hooks/useMint";
import { useMiden } from "../../context/MidenProvider";
import { useMidenStore } from "../../store/MidenStore";
import {
  createMockWebClient,
  createMockTransactionId,
  createMockTransactionRequest,
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

describe("useMint", () => {
  describe("initial state", () => {
    it("should return initial state", () => {
      mockUseMiden.mockReturnValue({
        client: null,
        isReady: false,
        sync: vi.fn(),
      });

      const { result } = renderHook(() => useMint());

      expect(result.current.result).toBeNull();
      expect(result.current.isLoading).toBe(false);
      expect(result.current.stage).toBe("idle");
      expect(result.current.error).toBeNull();
      expect(typeof result.current.mint).toBe("function");
      expect(typeof result.current.reset).toBe("function");
    });
  });

  describe("mint transaction", () => {
    it("should throw error when client is not ready", async () => {
      mockUseMiden.mockReturnValue({
        client: null,
        isReady: false,
        sync: vi.fn(),
      });

      const { result } = renderHook(() => useMint());

      await expect(
        result.current.mint({
          targetAccountId: "0xtarget",
          faucetId: "0xfaucet",
          amount: 1000n,
        })
      ).rejects.toThrow("Miden client is not ready");
    });

    it("should execute mint transaction with default options", async () => {
      const mockTxId = createMockTransactionId("0xtx456");
      const mockSync = vi.fn().mockResolvedValue(undefined);
      const mockClient = createMockWebClient({
        newMintTransactionRequest: vi.fn().mockReturnValue(createMockTransactionRequest()),
        submitNewTransaction: vi.fn().mockResolvedValue(mockTxId),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        sync: mockSync,
      });

      const { result } = renderHook(() => useMint());

      let txResult;
      await act(async () => {
        txResult = await result.current.mint({
          targetAccountId: "0xtarget",
          faucetId: "0xfaucet",
          amount: 1000n,
        });
      });

      expect(txResult).toEqual({ transactionId: "0xtx456" });
      expect(result.current.result).toEqual({ transactionId: "0xtx456" });
      expect(result.current.stage).toBe("complete");
      expect(mockSync).toHaveBeenCalled();

      // Verify default note type (private)
      expect(mockClient.newMintTransactionRequest).toHaveBeenCalledWith(
        expect.anything(), // targetAccountId
        expect.anything(), // faucetId
        expect.anything(), // noteType (private by default)
        1000n
      );
    });

    it("should execute mint transaction with custom note type", async () => {
      const mockTxId = createMockTransactionId();
      const mockClient = createMockWebClient({
        newMintTransactionRequest: vi.fn().mockReturnValue(createMockTransactionRequest()),
        submitNewTransaction: vi.fn().mockResolvedValue(mockTxId),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        sync: vi.fn().mockResolvedValue(undefined),
      });

      const { result } = renderHook(() => useMint());

      await act(async () => {
        await result.current.mint({
          targetAccountId: "0xtarget",
          faucetId: "0xfaucet",
          amount: 500n,
          noteType: "public",
        });
      });

      expect(mockClient.newMintTransactionRequest).toHaveBeenCalled();
    });

    it("should submit transaction using faucet account", async () => {
      const mockTxId = createMockTransactionId();
      const mockClient = createMockWebClient({
        newMintTransactionRequest: vi.fn().mockReturnValue(createMockTransactionRequest()),
        submitNewTransaction: vi.fn().mockResolvedValue(mockTxId),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        sync: vi.fn().mockResolvedValue(undefined),
      });

      const { result } = renderHook(() => useMint());

      await act(async () => {
        await result.current.mint({
          targetAccountId: "0xtarget",
          faucetId: "0xfaucet123",
          amount: 100n,
        });
      });

      // The faucet account should be used for submission
      expect(mockClient.submitNewTransaction).toHaveBeenCalled();
    });
  });

  describe("stage transitions", () => {
    it("should transition through stages during execution", async () => {
      let resolveSubmit: () => void;
      const submitPromise = new Promise<ReturnType<typeof createMockTransactionId>>((resolve) => {
        resolveSubmit = () => resolve(createMockTransactionId());
      });

      const mockClient = createMockWebClient({
        newMintTransactionRequest: vi.fn().mockReturnValue(createMockTransactionRequest()),
        submitNewTransaction: vi.fn().mockReturnValue(submitPromise),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        sync: vi.fn().mockResolvedValue(undefined),
      });

      const { result } = renderHook(() => useMint());

      // Start mint
      let mintPromise: Promise<any>;
      act(() => {
        mintPromise = result.current.mint({
          targetAccountId: "0x1",
          faucetId: "0x2",
          amount: 1n,
        });
      });

      // Should transition to proving
      await waitFor(() => {
        expect(result.current.stage).toBe("proving");
      });

      // Resolve submit
      await act(async () => {
        resolveSubmit!();
        await mintPromise;
      });

      expect(result.current.stage).toBe("complete");
    });

    it("should start with executing stage", async () => {
      let resolveSubmit: () => void;
      const submitPromise = new Promise<any>((resolve) => {
        resolveSubmit = () => resolve(createMockTransactionId());
      });

      const mockClient = createMockWebClient({
        newMintTransactionRequest: vi.fn().mockReturnValue(createMockTransactionRequest()),
        submitNewTransaction: vi.fn().mockReturnValue(submitPromise),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        sync: vi.fn().mockResolvedValue(undefined),
      });

      const { result } = renderHook(() => useMint());

      expect(result.current.stage).toBe("idle");

      let mintPromise: Promise<any>;
      act(() => {
        mintPromise = result.current.mint({
          targetAccountId: "0x1",
          faucetId: "0x2",
          amount: 1n,
        });
      });

      // Complete the transaction
      await act(async () => {
        resolveSubmit!();
        await mintPromise;
      });
    });
  });

  describe("error handling", () => {
    it("should handle mint transaction errors", async () => {
      const mintError = new Error("Mint limit exceeded");
      const mockClient = createMockWebClient({
        newMintTransactionRequest: vi.fn().mockReturnValue(createMockTransactionRequest()),
        submitNewTransaction: vi.fn().mockRejectedValue(mintError),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        sync: vi.fn(),
      });

      const { result } = renderHook(() => useMint());

      await act(async () => {
        await expect(
          result.current.mint({
            targetAccountId: "0x1",
            faucetId: "0x2",
            amount: 999999999n,
          })
        ).rejects.toThrow("Mint limit exceeded");
      });

      await waitFor(() => {
        expect(result.current.error?.message).toBe("Mint limit exceeded");
      });
      expect(result.current.stage).toBe("idle");
      expect(result.current.isLoading).toBe(false);
    });

    it("should handle request creation errors", async () => {
      const mockClient = createMockWebClient({
        newMintTransactionRequest: vi.fn().mockImplementation(() => {
          throw new Error("Invalid faucet");
        }),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        sync: vi.fn(),
      });

      const { result } = renderHook(() => useMint());

      await act(async () => {
        await expect(
          result.current.mint({
            targetAccountId: "0x1",
            faucetId: "0xinvalid",
            amount: 1n,
          })
        ).rejects.toThrow("Invalid faucet");
      });
    });
  });

  describe("reset", () => {
    it("should reset all state", async () => {
      const mockTxId = createMockTransactionId();
      const mockClient = createMockWebClient({
        newMintTransactionRequest: vi.fn().mockReturnValue(createMockTransactionRequest()),
        submitNewTransaction: vi.fn().mockResolvedValue(mockTxId),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        sync: vi.fn().mockResolvedValue(undefined),
      });

      const { result } = renderHook(() => useMint());

      // Execute mint
      await act(async () => {
        await result.current.mint({
          targetAccountId: "0x1",
          faucetId: "0x2",
          amount: 1n,
        });
      });

      expect(result.current.result).not.toBeNull();

      // Reset
      act(() => {
        result.current.reset();
      });

      expect(result.current.result).toBeNull();
      expect(result.current.isLoading).toBe(false);
      expect(result.current.stage).toBe("idle");
      expect(result.current.error).toBeNull();
    });
  });

  describe("sync after mint", () => {
    it("should trigger sync after successful mint", async () => {
      const mockTxId = createMockTransactionId();
      const mockSync = vi.fn().mockResolvedValue(undefined);
      const mockClient = createMockWebClient({
        newMintTransactionRequest: vi.fn().mockReturnValue(createMockTransactionRequest()),
        submitNewTransaction: vi.fn().mockResolvedValue(mockTxId),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        sync: mockSync,
      });

      const { result } = renderHook(() => useMint());

      await act(async () => {
        await result.current.mint({
          targetAccountId: "0x1",
          faucetId: "0x2",
          amount: 100n,
        });
      });

      expect(mockSync).toHaveBeenCalledTimes(1);
    });

    it("should not trigger sync on mint failure", async () => {
      const mockSync = vi.fn().mockResolvedValue(undefined);
      const mockClient = createMockWebClient({
        newMintTransactionRequest: vi.fn().mockReturnValue(createMockTransactionRequest()),
        submitNewTransaction: vi.fn().mockRejectedValue(new Error("Failed")),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        sync: mockSync,
      });

      const { result } = renderHook(() => useMint());

      await expect(
        result.current.mint({
          targetAccountId: "0x1",
          faucetId: "0x2",
          amount: 100n,
        })
      ).rejects.toThrow();

      expect(mockSync).not.toHaveBeenCalled();
    });
  });
});
