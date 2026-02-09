import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, act, waitFor } from "@testing-library/react";
import { useTransaction } from "../../hooks/useTransaction";
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

describe("useTransaction", () => {
  describe("initial state", () => {
    it("should return initial state", () => {
      mockUseMiden.mockReturnValue({
        client: null,
        isReady: false,
        sync: vi.fn(),
      });

      const { result } = renderHook(() => useTransaction());

      expect(result.current.result).toBeNull();
      expect(result.current.isLoading).toBe(false);
      expect(result.current.stage).toBe("idle");
      expect(result.current.error).toBeNull();
      expect(typeof result.current.execute).toBe("function");
      expect(typeof result.current.reset).toBe("function");
    });
  });

  describe("execute transaction", () => {
    it("should throw error when client is not ready", async () => {
      mockUseMiden.mockReturnValue({
        client: null,
        isReady: false,
        sync: vi.fn(),
      });

      const { result } = renderHook(() => useTransaction());

      await expect(
        result.current.execute({
          accountId: "0xaccount",
          request: createMockTransactionRequest(),
        })
      ).rejects.toThrow("Miden client is not ready");
    });

    it("should execute transaction with provided request", async () => {
      const mockTxId = createMockTransactionId("0xtx456");
      const mockSync = vi.fn().mockResolvedValue(undefined);
      const mockClient = createMockWebClient({
        submitNewTransaction: vi.fn().mockResolvedValue(mockTxId),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        sync: mockSync,
      });

      const { result } = renderHook(() => useTransaction());

      const request = createMockTransactionRequest();
      let txResult;
      await act(async () => {
        txResult = await result.current.execute({
          accountId: "0xaccount",
          request,
        });
      });

      expect(txResult).toEqual({ transactionId: "0xtx456" });
      expect(result.current.result).toEqual({ transactionId: "0xtx456" });
      expect(result.current.stage).toBe("complete");
      expect(mockClient.submitNewTransaction).toHaveBeenCalledWith(
        expect.anything(),
        request
      );
      expect(mockSync).toHaveBeenCalled();
    });

    it("should execute transaction with request factory", async () => {
      const mockTxId = createMockTransactionId("0xtx789");
      const mockClient = createMockWebClient({
        submitNewTransaction: vi.fn().mockResolvedValue(mockTxId),
      });
      const requestFactory = vi
        .fn()
        .mockResolvedValue(createMockTransactionRequest());

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        sync: vi.fn().mockResolvedValue(undefined),
      });

      const { result } = renderHook(() => useTransaction());

      await act(async () => {
        await result.current.execute({
          accountId: "0xaccount",
          request: requestFactory,
        });
      });

      expect(requestFactory).toHaveBeenCalledWith(mockClient);
      expect(mockClient.submitNewTransaction).toHaveBeenCalled();
    });
  });

  describe("stage transitions", () => {
    it("should transition through stages during execution", async () => {
      let resolveSubmit: () => void;
      const submitPromise = new Promise(
        (resolve) => (resolveSubmit = () => resolve(createMockTransactionId()))
      );

      const mockClient = createMockWebClient({
        submitNewTransaction: vi.fn().mockReturnValue(submitPromise),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        sync: vi.fn().mockResolvedValue(undefined),
      });

      const { result } = renderHook(() => useTransaction());

      let execPromise: Promise<any>;
      act(() => {
        execPromise = result.current.execute({
          accountId: "0x1",
          request: createMockTransactionRequest(),
        });
      });

      await waitFor(() => {
        expect(result.current.stage).toBe("proving");
      });

      await act(async () => {
        resolveSubmit!();
        await execPromise;
      });

      expect(result.current.stage).toBe("complete");
    });
  });

  describe("error handling", () => {
    it("should handle execution errors", async () => {
      const execError = new Error("Execution failed");
      const mockClient = createMockWebClient({
        submitNewTransaction: vi.fn().mockRejectedValue(execError),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        sync: vi.fn(),
      });

      const { result } = renderHook(() => useTransaction());

      await act(async () => {
        await expect(
          result.current.execute({
            accountId: "0x1",
            request: createMockTransactionRequest(),
          })
        ).rejects.toThrow("Execution failed");
      });

      await waitFor(() => {
        expect(result.current.error?.message).toBe("Execution failed");
      });
      expect(result.current.stage).toBe("idle");
      expect(result.current.isLoading).toBe(false);
    });
  });
});
