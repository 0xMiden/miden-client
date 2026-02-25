import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, act, waitFor } from "@testing-library/react";
import { useSwap } from "../../hooks/useSwap";
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

describe("useSwap", () => {
  describe("initial state", () => {
    it("should return initial state", () => {
      mockUseMiden.mockReturnValue({
        client: null,
        isReady: false,
        sync: vi.fn(),
      });

      const { result } = renderHook(() => useSwap());

      expect(result.current.result).toBeNull();
      expect(result.current.isLoading).toBe(false);
      expect(result.current.stage).toBe("idle");
      expect(result.current.error).toBeNull();
      expect(typeof result.current.swap).toBe("function");
      expect(typeof result.current.reset).toBe("function");
    });
  });

  describe("swap transaction", () => {
    it("should throw error when client is not ready", async () => {
      mockUseMiden.mockReturnValue({
        client: null,
        isReady: false,
        sync: vi.fn(),
      });

      const { result } = renderHook(() => useSwap());

      await expect(
        result.current.swap({
          accountId: "0xaccount",
          offeredFaucetId: "0xfaucetA",
          offeredAmount: 100n,
          requestedFaucetId: "0xfaucetB",
          requestedAmount: 50n,
        })
      ).rejects.toThrow("Miden client is not ready");
    });

    it("should execute swap transaction with default options", async () => {
      const mockTxId = createMockTransactionId("0xswaptx");
      const mockSync = vi.fn().mockResolvedValue(undefined);
      const mockClient = createMockWebClient({
        newSwapTransactionRequest: vi
          .fn()
          .mockReturnValue(createMockTransactionRequest()),
        submitNewTransaction: vi.fn().mockResolvedValue(mockTxId),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        sync: mockSync,
      });

      const { result } = renderHook(() => useSwap());

      let txResult;
      await act(async () => {
        txResult = await result.current.swap({
          accountId: "0xaccount",
          offeredFaucetId: "0xfaucetA",
          offeredAmount: 100n,
          requestedFaucetId: "0xfaucetB",
          requestedAmount: 50n,
        });
      });

      expect(txResult).toEqual({ transactionId: "0xswaptx" });
      expect(result.current.result).toEqual({ transactionId: "0xswaptx" });
      expect(result.current.stage).toBe("complete");
      expect(mockSync).toHaveBeenCalled();

      // Verify default note types (private)
      expect(mockClient.newSwapTransactionRequest).toHaveBeenCalledWith(
        expect.anything(), // accountId
        expect.anything(), // offeredFaucetId
        100n,
        expect.anything(), // requestedFaucetId
        50n,
        expect.anything(), // noteType (private)
        expect.anything() // paybackNoteType (private)
      );
    });

    it("should execute swap transaction with custom note types", async () => {
      const mockTxId = createMockTransactionId();
      const mockClient = createMockWebClient({
        newSwapTransactionRequest: vi
          .fn()
          .mockReturnValue(createMockTransactionRequest()),
        submitNewTransaction: vi.fn().mockResolvedValue(mockTxId),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        sync: vi.fn().mockResolvedValue(undefined),
      });

      const { result } = renderHook(() => useSwap());

      await act(async () => {
        await result.current.swap({
          accountId: "0xaccount",
          offeredFaucetId: "0xfaucetA",
          offeredAmount: 200n,
          requestedFaucetId: "0xfaucetB",
          requestedAmount: 100n,
          noteType: "public",
          paybackNoteType: "public",
        });
      });

      expect(mockClient.newSwapTransactionRequest).toHaveBeenCalled();
    });

    it("should handle different note type combinations", async () => {
      const mockTxId = createMockTransactionId();
      const mockClient = createMockWebClient({
        newSwapTransactionRequest: vi
          .fn()
          .mockReturnValue(createMockTransactionRequest()),
        submitNewTransaction: vi.fn().mockResolvedValue(mockTxId),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        sync: vi.fn().mockResolvedValue(undefined),
      });

      const { result } = renderHook(() => useSwap());

      // Private / Private
      await act(async () => {
        await result.current.swap({
          accountId: "0x1",
          offeredFaucetId: "0xA",
          offeredAmount: 1n,
          requestedFaucetId: "0xB",
          requestedAmount: 1n,
          noteType: "private",
          paybackNoteType: "private",
        });
      });

      // Public / Public
      act(() => {
        result.current.reset();
      });
      await act(async () => {
        await result.current.swap({
          accountId: "0x1",
          offeredFaucetId: "0xA",
          offeredAmount: 1n,
          requestedFaucetId: "0xB",
          requestedAmount: 1n,
          noteType: "public",
          paybackNoteType: "public",
        });
      });

      expect(mockClient.newSwapTransactionRequest).toHaveBeenCalledTimes(2);
    });
  });

  describe("stage transitions", () => {
    it("should transition through stages during execution", async () => {
      let resolveSubmit: () => void;
      const submitPromise = new Promise<
        ReturnType<typeof createMockTransactionId>
      >((resolve) => {
        resolveSubmit = () => resolve(createMockTransactionId());
      });

      const mockClient = createMockWebClient({
        newSwapTransactionRequest: vi
          .fn()
          .mockReturnValue(createMockTransactionRequest()),
        submitNewTransaction: vi.fn().mockReturnValue(submitPromise),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        sync: vi.fn().mockResolvedValue(undefined),
      });

      const { result } = renderHook(() => useSwap());

      // Start swap
      let swapPromise: Promise<any>;
      act(() => {
        swapPromise = result.current.swap({
          accountId: "0x1",
          offeredFaucetId: "0xA",
          offeredAmount: 100n,
          requestedFaucetId: "0xB",
          requestedAmount: 50n,
        });
      });

      // Should be in proving stage
      await waitFor(() => {
        expect(result.current.stage).toBe("proving");
      });

      // Resolve submit
      await act(async () => {
        resolveSubmit!();
        await swapPromise;
      });

      expect(result.current.stage).toBe("complete");
    });
  });

  describe("error handling", () => {
    it("should handle swap transaction errors", async () => {
      const swapError = new Error("Insufficient liquidity");
      const mockClient = createMockWebClient({
        newSwapTransactionRequest: vi
          .fn()
          .mockReturnValue(createMockTransactionRequest()),
        submitNewTransaction: vi.fn().mockRejectedValue(swapError),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        sync: vi.fn(),
      });

      const { result } = renderHook(() => useSwap());

      await act(async () => {
        await expect(
          result.current.swap({
            accountId: "0x1",
            offeredFaucetId: "0xA",
            offeredAmount: 1000000n,
            requestedFaucetId: "0xB",
            requestedAmount: 1n,
          })
        ).rejects.toThrow("Insufficient liquidity");
      });

      await waitFor(() => {
        expect(result.current.error?.message).toBe("Insufficient liquidity");
      });
      expect(result.current.stage).toBe("idle");
      expect(result.current.isLoading).toBe(false);
    });

    it("should handle request creation errors", async () => {
      const mockClient = createMockWebClient({
        newSwapTransactionRequest: vi.fn().mockImplementation(() => {
          throw new Error("Invalid swap parameters");
        }),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        sync: vi.fn(),
      });

      const { result } = renderHook(() => useSwap());

      await act(async () => {
        await expect(
          result.current.swap({
            accountId: "0x1",
            offeredFaucetId: "0xA",
            offeredAmount: 0n, // Invalid
            requestedFaucetId: "0xB",
            requestedAmount: 0n, // Invalid
          })
        ).rejects.toThrow("Invalid swap parameters");
      });
    });

    it("should handle same asset swap error", async () => {
      const mockClient = createMockWebClient({
        newSwapTransactionRequest: vi.fn().mockImplementation(() => {
          throw new Error("Cannot swap same asset");
        }),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        sync: vi.fn(),
      });

      const { result } = renderHook(() => useSwap());

      await act(async () => {
        await expect(
          result.current.swap({
            accountId: "0x1",
            offeredFaucetId: "0xsamefaucet",
            offeredAmount: 100n,
            requestedFaucetId: "0xsamefaucet",
            requestedAmount: 100n,
          })
        ).rejects.toThrow("Cannot swap same asset");
      });
    });
  });

  describe("reset", () => {
    it("should reset all state", async () => {
      const mockTxId = createMockTransactionId();
      const mockClient = createMockWebClient({
        newSwapTransactionRequest: vi
          .fn()
          .mockReturnValue(createMockTransactionRequest()),
        submitNewTransaction: vi.fn().mockResolvedValue(mockTxId),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        sync: vi.fn().mockResolvedValue(undefined),
      });

      const { result } = renderHook(() => useSwap());

      // Execute swap
      await act(async () => {
        await result.current.swap({
          accountId: "0x1",
          offeredFaucetId: "0xA",
          offeredAmount: 100n,
          requestedFaucetId: "0xB",
          requestedAmount: 50n,
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

  describe("loading state", () => {
    it("should track loading state during swap", async () => {
      let resolvePromise: (value: any) => void;
      const submitPromise = new Promise((resolve) => {
        resolvePromise = resolve;
      });

      const mockClient = createMockWebClient({
        newSwapTransactionRequest: vi
          .fn()
          .mockReturnValue(createMockTransactionRequest()),
        submitNewTransaction: vi.fn().mockReturnValue(submitPromise),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        sync: vi.fn().mockResolvedValue(undefined),
      });

      const { result } = renderHook(() => useSwap());

      // Start swap
      let swapPromise: Promise<any>;
      act(() => {
        swapPromise = result.current.swap({
          accountId: "0x1",
          offeredFaucetId: "0xA",
          offeredAmount: 100n,
          requestedFaucetId: "0xB",
          requestedAmount: 50n,
        });
      });

      // Should be loading
      await waitFor(() => {
        expect(result.current.isLoading).toBe(true);
      });

      // Resolve
      await act(async () => {
        resolvePromise!(createMockTransactionId());
        await swapPromise;
      });

      expect(result.current.isLoading).toBe(false);
    });
  });

  describe("sync after swap", () => {
    it("should trigger sync after successful swap", async () => {
      const mockTxId = createMockTransactionId();
      const mockSync = vi.fn().mockResolvedValue(undefined);
      const mockClient = createMockWebClient({
        newSwapTransactionRequest: vi
          .fn()
          .mockReturnValue(createMockTransactionRequest()),
        submitNewTransaction: vi.fn().mockResolvedValue(mockTxId),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        sync: mockSync,
      });

      const { result } = renderHook(() => useSwap());

      await act(async () => {
        await result.current.swap({
          accountId: "0x1",
          offeredFaucetId: "0xA",
          offeredAmount: 100n,
          requestedFaucetId: "0xB",
          requestedAmount: 50n,
        });
      });

      expect(mockSync).toHaveBeenCalledTimes(1);
    });

    it("should not trigger sync on swap failure", async () => {
      const mockSync = vi.fn().mockResolvedValue(undefined);
      const mockClient = createMockWebClient({
        newSwapTransactionRequest: vi
          .fn()
          .mockReturnValue(createMockTransactionRequest()),
        submitNewTransaction: vi.fn().mockRejectedValue(new Error("Failed")),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        sync: mockSync,
      });

      const { result } = renderHook(() => useSwap());

      await act(async () => {
        await expect(
          result.current.swap({
            accountId: "0x1",
            offeredFaucetId: "0xA",
            offeredAmount: 100n,
            requestedFaucetId: "0xB",
            requestedAmount: 50n,
          })
        ).rejects.toThrow();
      });

      expect(mockSync).not.toHaveBeenCalled();
    });
  });

  describe("bigint handling", () => {
    it("should handle large amounts correctly", async () => {
      const mockTxId = createMockTransactionId();
      const mockClient = createMockWebClient({
        newSwapTransactionRequest: vi
          .fn()
          .mockReturnValue(createMockTransactionRequest()),
        submitNewTransaction: vi.fn().mockResolvedValue(mockTxId),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        sync: vi.fn().mockResolvedValue(undefined),
      });

      const { result } = renderHook(() => useSwap());

      const largeOfferedAmount = 1000000000000000000n;
      const largeRequestedAmount = 500000000000000000n;

      await act(async () => {
        await result.current.swap({
          accountId: "0x1",
          offeredFaucetId: "0xA",
          offeredAmount: largeOfferedAmount,
          requestedFaucetId: "0xB",
          requestedAmount: largeRequestedAmount,
        });
      });

      expect(mockClient.newSwapTransactionRequest).toHaveBeenCalledWith(
        expect.anything(),
        expect.anything(),
        largeOfferedAmount,
        expect.anything(),
        largeRequestedAmount,
        expect.anything(),
        expect.anything()
      );
    });
  });
});
