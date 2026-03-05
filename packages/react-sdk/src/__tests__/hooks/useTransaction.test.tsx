import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, act, waitFor } from "@testing-library/react";
import { useTransaction } from "../../hooks/useTransaction";
import { useMiden } from "../../context/MidenProvider";
import { useMidenStore } from "../../store/MidenStore";
import { NoteType } from "@miden-sdk/miden-sdk";
import {
  createMockWebClient,
  createMockTransactionId,
  createMockTransactionRequest,
  createMockNote,
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

  describe("concurrency guard", () => {
    it("should reject concurrent executions with SEND_BUSY", async () => {
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

      let firstExec: Promise<any>;
      act(() => {
        firstExec = result.current.execute({
          accountId: "0x1",
          request: createMockTransactionRequest(),
        });
      });

      await expect(
        result.current.execute({
          accountId: "0x1",
          request: createMockTransactionRequest(),
        })
      ).rejects.toThrow("A transaction is already in progress");

      await act(async () => {
        resolveSubmit!();
        await firstExec;
      });
    });
  });

  describe("auto-sync", () => {
    it("should call sync before execute by default", async () => {
      const mockSync = vi.fn().mockResolvedValue(undefined);
      const mockTxId = createMockTransactionId();
      const mockClient = createMockWebClient({
        submitNewTransaction: vi.fn().mockResolvedValue(mockTxId),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        sync: mockSync,
      });

      const { result } = renderHook(() => useTransaction());

      await act(async () => {
        await result.current.execute({
          accountId: "0x1",
          request: createMockTransactionRequest(),
        });
      });

      expect(mockSync).toHaveBeenCalled();
    });

    it("should skip pre-sync when skipSync is true", async () => {
      const mockSync = vi.fn().mockResolvedValue(undefined);
      const mockTxId = createMockTransactionId();
      const mockClient = createMockWebClient({
        submitNewTransaction: vi.fn().mockResolvedValue(mockTxId),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        sync: mockSync,
      });

      const { result } = renderHook(() => useTransaction());

      await act(async () => {
        await result.current.execute({
          accountId: "0x1",
          request: createMockTransactionRequest(),
          skipSync: true,
        });
      });

      // Sync called only once (post-execute), not before
      expect(mockSync).toHaveBeenCalledTimes(1);
    });
  });

  describe("privateNoteRecipient (4-step pipeline)", () => {
    const createMockTxResultWithPrivateNotes = (
      id: string = "0xtx_private"
    ) => {
      const mockNote = createMockNote("0xprivate_note");
      return {
        id: vi.fn(() => createMockTransactionId(id)),
        executedTransaction: vi.fn(() => ({
          outputNotes: vi.fn(() => ({
            notes: vi.fn(() => [
              {
                noteType: vi.fn(() => NoteType.Private),
                intoFull: vi.fn(() => mockNote),
              },
            ]),
          })),
        })),
        serialize: vi.fn(() => new Uint8Array()),
      };
    };

    it("should use 4-step pipeline when privateNoteRecipient is set", async () => {
      const mockTxResult = createMockTxResultWithPrivateNotes("0xtx_4step");
      const mockSync = vi.fn().mockResolvedValue(undefined);
      const record = {
        id: vi.fn(() => ({ toHex: () => "0xtx_4step" })),
        transactionStatus: vi.fn(() => ({
          isPending: vi.fn(() => false),
          isCommitted: vi.fn(() => true),
          isDiscarded: vi.fn(() => false),
        })),
      };
      const mockClient = createMockWebClient({
        executeTransaction: vi.fn().mockResolvedValue(mockTxResult),
        proveTransaction: vi.fn().mockResolvedValue({}),
        submitProvenTransaction: vi.fn().mockResolvedValue(100),
        applyTransaction: vi.fn().mockResolvedValue({}),
        getTransactions: vi.fn().mockResolvedValue([record]),
        sendPrivateNote: vi.fn().mockResolvedValue(undefined),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        sync: mockSync,
      });

      const { result } = renderHook(() => useTransaction());

      let txResult: any;
      await act(async () => {
        txResult = await result.current.execute({
          accountId: "0xaccount",
          request: createMockTransactionRequest(),
          privateNoteRecipient: "0xrecipient",
        });
      });

      expect(txResult.transactionId).toBe("0xtx_4step");
      expect(mockClient.executeTransaction).toHaveBeenCalled();
      expect(mockClient.proveTransaction).toHaveBeenCalled();
      expect(mockClient.submitProvenTransaction).toHaveBeenCalled();
      expect(mockClient.applyTransaction).toHaveBeenCalled();
      expect(mockClient.submitNewTransaction).not.toHaveBeenCalled();
      expect(result.current.stage).toBe("complete");
      expect(mockSync).toHaveBeenCalled();
    });

    it("should deliver private notes to recipient", async () => {
      const mockTxResult = createMockTxResultWithPrivateNotes();
      const record = {
        id: vi.fn(() => ({ toHex: () => "0xtx_private" })),
        transactionStatus: vi.fn(() => ({
          isPending: vi.fn(() => false),
          isCommitted: vi.fn(() => true),
          isDiscarded: vi.fn(() => false),
        })),
      };
      const mockClient = createMockWebClient({
        executeTransaction: vi.fn().mockResolvedValue(mockTxResult),
        proveTransaction: vi.fn().mockResolvedValue({}),
        submitProvenTransaction: vi.fn().mockResolvedValue(100),
        applyTransaction: vi.fn().mockResolvedValue({}),
        getTransactions: vi.fn().mockResolvedValue([record]),
        sendPrivateNote: vi.fn().mockResolvedValue(undefined),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        sync: vi.fn().mockResolvedValue(undefined),
      });

      const { result } = renderHook(() => useTransaction());

      await act(async () => {
        await result.current.execute({
          accountId: "0xaccount",
          request: createMockTransactionRequest(),
          privateNoteRecipient: "0xrecipient",
        });
      });

      expect(mockClient.sendPrivateNote).toHaveBeenCalledTimes(1);
      expect(mockClient.sendPrivateNote).toHaveBeenCalledWith(
        expect.anything(), // full note
        expect.anything() // recipient address
      );
    });

    it("should not deliver notes when there are no private output notes", async () => {
      const mockTxResult = {
        id: vi.fn(() => createMockTransactionId("0xtx_noprivate")),
        executedTransaction: vi.fn(() => ({
          outputNotes: vi.fn(() => ({
            notes: vi.fn(() => [
              {
                noteType: vi.fn(() => NoteType.Public),
                intoFull: vi.fn(() => null),
              },
            ]),
          })),
        })),
        serialize: vi.fn(() => new Uint8Array()),
      };

      const record = {
        id: vi.fn(() => ({ toHex: () => "0xtx_noprivate" })),
        transactionStatus: vi.fn(() => ({
          isPending: vi.fn(() => false),
          isCommitted: vi.fn(() => true),
          isDiscarded: vi.fn(() => false),
        })),
      };
      const mockClient = createMockWebClient({
        executeTransaction: vi.fn().mockResolvedValue(mockTxResult),
        proveTransaction: vi.fn().mockResolvedValue({}),
        submitProvenTransaction: vi.fn().mockResolvedValue(100),
        applyTransaction: vi.fn().mockResolvedValue({}),
        getTransactions: vi.fn().mockResolvedValue([record]),
        sendPrivateNote: vi.fn().mockResolvedValue(undefined),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        sync: vi.fn().mockResolvedValue(undefined),
      });

      const { result } = renderHook(() => useTransaction());

      await act(async () => {
        await result.current.execute({
          accountId: "0xaccount",
          request: createMockTransactionRequest(),
          privateNoteRecipient: "0xrecipient",
        });
      });

      expect(mockClient.sendPrivateNote).not.toHaveBeenCalled();
    });

    it("should handle errors in 4-step pipeline", async () => {
      const mockClient = createMockWebClient({
        executeTransaction: vi
          .fn()
          .mockRejectedValue(new Error("Execute failed")),
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
            privateNoteRecipient: "0xrecipient",
          })
        ).rejects.toThrow("Execute failed");
      });

      await waitFor(() => {
        expect(result.current.error?.message).toBe("Execute failed");
      });
      expect(result.current.stage).toBe("idle");
    });
  });
});
