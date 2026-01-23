import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, act, waitFor } from "@testing-library/react";
import { useSend } from "../../hooks/useSend";
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

describe("useSend", () => {
  describe("initial state", () => {
    it("should return initial state", () => {
      mockUseMiden.mockReturnValue({
        client: null,
        isReady: false,
        sync: vi.fn(),
      });

      const { result } = renderHook(() => useSend());

      expect(result.current.result).toBeNull();
      expect(result.current.isLoading).toBe(false);
      expect(result.current.stage).toBe("idle");
      expect(result.current.error).toBeNull();
      expect(typeof result.current.send).toBe("function");
      expect(typeof result.current.reset).toBe("function");
    });
  });

  describe("send transaction", () => {
    it("should throw error when client is not ready", async () => {
      mockUseMiden.mockReturnValue({
        client: null,
        isReady: false,
        sync: vi.fn(),
      });

      const { result } = renderHook(() => useSend());

      await expect(
        result.current.send({
          from: "0xsender",
          to: "0xrecipient",
          faucetId: "0xfaucet",
          amount: 100n,
        })
      ).rejects.toThrow("Miden client is not ready");
    });

    it("should execute send transaction with default options", async () => {
      const mockTxId = createMockTransactionId("0xtx123");
      const mockSync = vi.fn().mockResolvedValue(undefined);
      const mockClient = createMockWebClient({
        newSendTransactionRequest: vi.fn().mockReturnValue(createMockTransactionRequest()),
        submitNewTransaction: vi.fn().mockResolvedValue(mockTxId),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        sync: mockSync,
      });

      const { result } = renderHook(() => useSend());

      let txResult;
      await act(async () => {
        txResult = await result.current.send({
          from: "0xsender",
          to: "0xrecipient",
          faucetId: "0xfaucet",
          amount: 100n,
        });
      });

      expect(txResult).toEqual({ transactionId: "0xtx123" });
      expect(result.current.result).toEqual({ transactionId: "0xtx123" });
      expect(result.current.stage).toBe("complete");
      expect(result.current.isLoading).toBe(false);
      expect(mockSync).toHaveBeenCalled();
    });

    it("should execute send transaction with custom options", async () => {
      const mockTxId = createMockTransactionId();
      const mockSync = vi.fn().mockResolvedValue(undefined);
      const mockClient = createMockWebClient({
        newSendTransactionRequest: vi.fn().mockReturnValue(createMockTransactionRequest()),
        submitNewTransaction: vi.fn().mockResolvedValue(mockTxId),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        sync: mockSync,
      });

      const { result } = renderHook(() => useSend());

      await act(async () => {
        await result.current.send({
          from: "0xsender",
          to: "0xrecipient",
          faucetId: "0xfaucet",
          amount: 500n,
          noteType: "public",
          recallHeight: 1000,
          timelockHeight: 500,
        });
      });

      expect(mockClient.newSendTransactionRequest).toHaveBeenCalledWith(
        expect.anything(), // fromAccountId
        expect.anything(), // toAccountId
        expect.anything(), // faucetIdObj
        expect.anything(), // noteType (public)
        500n,
        1000,
        500
      );
    });

    it("should handle different note types", async () => {
      const mockTxId = createMockTransactionId();
      const mockSync = vi.fn().mockResolvedValue(undefined);
      const mockClient = createMockWebClient({
        newSendTransactionRequest: vi.fn().mockReturnValue(createMockTransactionRequest()),
        submitNewTransaction: vi.fn().mockResolvedValue(mockTxId),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        sync: mockSync,
      });

      const { result } = renderHook(() => useSend());

      // Test private
      await act(async () => {
        await result.current.send({
          from: "0x1",
          to: "0x2",
          faucetId: "0x3",
          amount: 1n,
          noteType: "private",
        });
      });

      // Test encrypted
      result.current.reset();
      await act(async () => {
        await result.current.send({
          from: "0x1",
          to: "0x2",
          faucetId: "0x3",
          amount: 1n,
          noteType: "encrypted",
        });
      });

      expect(mockClient.newSendTransactionRequest).toHaveBeenCalledTimes(2);
    });
  });

  describe("stage transitions", () => {
    it("should transition through stages during execution", async () => {
      let resolveSubmit: () => void;

      const submitPromise = new Promise<ReturnType<typeof createMockTransactionId>>((resolve) => {
        resolveSubmit = () => resolve(createMockTransactionId());
      });

      const mockClient = createMockWebClient({
        newSendTransactionRequest: vi.fn().mockImplementation(() => {
          return createMockTransactionRequest();
        }),
        submitNewTransaction: vi.fn().mockReturnValue(submitPromise),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        sync: vi.fn().mockResolvedValue(undefined),
      });

      const { result } = renderHook(() => useSend());

      // Start send
      let sendPromise: Promise<any>;
      act(() => {
        sendPromise = result.current.send({
          from: "0x1",
          to: "0x2",
          faucetId: "0x3",
          amount: 1n,
        });
      });

      // Should transition to executing
      await waitFor(() => {
        expect(result.current.stage).toBe("proving");
      });

      // Resolve submit
      await act(async () => {
        resolveSubmit!();
        await sendPromise;
      });

      expect(result.current.stage).toBe("complete");
    });
  });

  describe("error handling", () => {
    it("should handle transaction errors", async () => {
      const txError = new Error("Insufficient balance");
      const mockClient = createMockWebClient({
        newSendTransactionRequest: vi.fn().mockReturnValue(createMockTransactionRequest()),
        submitNewTransaction: vi.fn().mockRejectedValue(txError),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        sync: vi.fn(),
      });

      const { result } = renderHook(() => useSend());

      await act(async () => {
        await expect(
          result.current.send({
            from: "0x1",
            to: "0x2",
            faucetId: "0x3",
            amount: 1000000n,
          })
        ).rejects.toThrow("Insufficient balance");
      });

      await waitFor(() => {
        expect(result.current.error?.message).toBe("Insufficient balance");
      });
      expect(result.current.stage).toBe("idle");
      expect(result.current.isLoading).toBe(false);
    });

    it("should handle request creation errors", async () => {
      const mockClient = createMockWebClient({
        newSendTransactionRequest: vi.fn().mockImplementation(() => {
          throw new Error("Invalid parameters");
        }),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        sync: vi.fn(),
      });

      const { result } = renderHook(() => useSend());

      await act(async () => {
        await expect(
          result.current.send({
            from: "0x1",
            to: "0x2",
            faucetId: "0x3",
            amount: 1n,
          })
        ).rejects.toThrow("Invalid parameters");
      });

      await waitFor(() => {
        expect(result.current.error).not.toBeNull();
      });
    });
  });

  describe("reset", () => {
    it("should reset all state", async () => {
      const mockTxId = createMockTransactionId();
      const mockClient = createMockWebClient({
        newSendTransactionRequest: vi.fn().mockReturnValue(createMockTransactionRequest()),
        submitNewTransaction: vi.fn().mockResolvedValue(mockTxId),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        sync: vi.fn().mockResolvedValue(undefined),
      });

      const { result } = renderHook(() => useSend());

      // Execute send
      await act(async () => {
        await result.current.send({
          from: "0x1",
          to: "0x2",
          faucetId: "0x3",
          amount: 1n,
        });
      });

      expect(result.current.result).not.toBeNull();
      expect(result.current.stage).toBe("complete");

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
    it("should set isLoading during transaction", async () => {
      let resolvePromise: (value: any) => void;
      const submitPromise = new Promise((resolve) => {
        resolvePromise = resolve;
      });

      const mockClient = createMockWebClient({
        newSendTransactionRequest: vi.fn().mockReturnValue(createMockTransactionRequest()),
        submitNewTransaction: vi.fn().mockReturnValue(submitPromise),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        sync: vi.fn().mockResolvedValue(undefined),
      });

      const { result } = renderHook(() => useSend());

      // Start send
      let sendPromise: Promise<any>;
      act(() => {
        sendPromise = result.current.send({
          from: "0x1",
          to: "0x2",
          faucetId: "0x3",
          amount: 1n,
        });
      });

      // Should be loading
      await waitFor(() => {
        expect(result.current.isLoading).toBe(true);
      });

      // Resolve
      await act(async () => {
        resolvePromise!(createMockTransactionId());
        await sendPromise;
      });

      expect(result.current.isLoading).toBe(false);
    });
  });
});
