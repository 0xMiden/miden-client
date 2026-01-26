import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, act, waitFor } from "@testing-library/react";
import { useConsume } from "../../hooks/useConsume";
import { useMiden } from "../../context/MidenProvider";
import { useMidenStore } from "../../store/MidenStore";
import {
  createMockInputNoteRecord,
  createMockWebClient,
  createMockTransactionId,
  createMockTransactionRequest,
} from "../mocks/miden-sdk";

// Mock useMiden
vi.mock("../../context/MidenProvider", () => ({
  useMiden: vi.fn(),
}));

const mockUseMiden = useMiden as ReturnType<typeof vi.fn>;

const createNoteRecords = (noteIds: string[]) =>
  noteIds.map((noteId) => createMockInputNoteRecord(noteId));

beforeEach(() => {
  useMidenStore.getState().reset();
  vi.clearAllMocks();
});

describe("useConsume", () => {
  describe("initial state", () => {
    it("should return initial state", () => {
      mockUseMiden.mockReturnValue({
        client: null,
        isReady: false,
        sync: vi.fn(),
      });

      const { result } = renderHook(() => useConsume());

      expect(result.current.result).toBeNull();
      expect(result.current.isLoading).toBe(false);
      expect(result.current.stage).toBe("idle");
      expect(result.current.error).toBeNull();
      expect(typeof result.current.consume).toBe("function");
      expect(typeof result.current.reset).toBe("function");
    });
  });

  describe("consume transaction", () => {
    it("should throw error when client is not ready", async () => {
      mockUseMiden.mockReturnValue({
        client: null,
        isReady: false,
        sync: vi.fn(),
      });

      const { result } = renderHook(() => useConsume());

      await expect(
        result.current.consume({
          accountId: "0xaccount",
          noteIds: ["0xnote1", "0xnote2"],
        })
      ).rejects.toThrow("Miden client is not ready");
    });

    it("should throw error when no note IDs provided", async () => {
      const mockClient = createMockWebClient();

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        sync: vi.fn(),
      });

      const { result } = renderHook(() => useConsume());

      await expect(
        result.current.consume({
          accountId: "0xaccount",
          noteIds: [],
        })
      ).rejects.toThrow("No note IDs provided");
    });

    it("should execute consume transaction", async () => {
      const mockTxId = createMockTransactionId("0xtx789");
      const mockSync = vi.fn().mockResolvedValue(undefined);
      const noteIds = ["0xnote1", "0xnote2"];
      const noteRecords = createNoteRecords(noteIds);
      const mockClient = createMockWebClient({
        getInputNotes: vi.fn().mockResolvedValue(noteRecords),
        newConsumeTransactionRequest: vi
          .fn()
          .mockReturnValue(createMockTransactionRequest()),
        submitNewTransaction: vi.fn().mockResolvedValue(mockTxId),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        sync: mockSync,
      });

      const { result } = renderHook(() => useConsume());

      let txResult;
      await act(async () => {
        txResult = await result.current.consume({
          accountId: "0xaccount",
          noteIds,
        });
      });

      expect(txResult).toEqual({ transactionId: "0xtx789" });
      expect(result.current.result).toEqual({ transactionId: "0xtx789" });
      expect(result.current.stage).toBe("complete");
      expect(mockSync).toHaveBeenCalled();

      // Verify notes were passed
      const [notes] = mockClient.newConsumeTransactionRequest.mock.calls[0];
      expect(notes.map((note) => note.id().toString())).toEqual(noteIds);
    });

    it("should consume single note", async () => {
      const mockTxId = createMockTransactionId();
      const noteIds = ["0xsinglenote"];
      const noteRecords = createNoteRecords(noteIds);
      const mockClient = createMockWebClient({
        getInputNotes: vi.fn().mockResolvedValue(noteRecords),
        newConsumeTransactionRequest: vi
          .fn()
          .mockReturnValue(createMockTransactionRequest()),
        submitNewTransaction: vi.fn().mockResolvedValue(mockTxId),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        sync: vi.fn().mockResolvedValue(undefined),
      });

      const { result } = renderHook(() => useConsume());

      await act(async () => {
        await result.current.consume({
          accountId: "0xaccount",
          noteIds,
        });
      });

      const [notes] = mockClient.newConsumeTransactionRequest.mock.calls[0];
      expect(notes.map((note) => note.id().toString())).toEqual(noteIds);
    });

    it("should consume multiple notes in one transaction", async () => {
      const mockTxId = createMockTransactionId();
      const noteIds = ["0xnote1", "0xnote2", "0xnote3", "0xnote4", "0xnote5"];
      const noteRecords = createNoteRecords(noteIds);
      const mockClient = createMockWebClient({
        getInputNotes: vi.fn().mockResolvedValue(noteRecords),
        newConsumeTransactionRequest: vi
          .fn()
          .mockReturnValue(createMockTransactionRequest()),
        submitNewTransaction: vi.fn().mockResolvedValue(mockTxId),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        sync: vi.fn().mockResolvedValue(undefined),
      });

      const { result } = renderHook(() => useConsume());

      await act(async () => {
        await result.current.consume({
          accountId: "0xaccount",
          noteIds,
        });
      });

      const [notes] = mockClient.newConsumeTransactionRequest.mock.calls[0];
      expect(notes.map((note) => note.id().toString())).toEqual(noteIds);
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

      const noteRecords = createNoteRecords(["0xnote1"]);
      const mockClient = createMockWebClient({
        getInputNotes: vi.fn().mockResolvedValue(noteRecords),
        newConsumeTransactionRequest: vi
          .fn()
          .mockReturnValue(createMockTransactionRequest()),
        submitNewTransaction: vi.fn().mockReturnValue(submitPromise),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        sync: vi.fn().mockResolvedValue(undefined),
      });

      const { result } = renderHook(() => useConsume());

      // Start consume
      let consumePromise: Promise<any>;
      act(() => {
        consumePromise = result.current.consume({
          accountId: "0x1",
          noteIds: ["0xnote1"],
        });
      });

      // Should be in proving stage
      await waitFor(() => {
        expect(result.current.stage).toBe("proving");
      });

      // Resolve submit
      await act(async () => {
        resolveSubmit!();
        await consumePromise;
      });

      expect(result.current.stage).toBe("complete");
    });
  });

  describe("error handling", () => {
    it("should handle consume transaction errors", async () => {
      const consumeError = new Error("Note already consumed");
      const noteRecords = createNoteRecords(["0xconsumed"]);
      const mockClient = createMockWebClient({
        getInputNotes: vi.fn().mockResolvedValue(noteRecords),
        newConsumeTransactionRequest: vi
          .fn()
          .mockReturnValue(createMockTransactionRequest()),
        submitNewTransaction: vi.fn().mockRejectedValue(consumeError),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        sync: vi.fn(),
      });

      const { result } = renderHook(() => useConsume());

      await act(async () => {
        await expect(
          result.current.consume({
            accountId: "0x1",
            noteIds: ["0xconsumed"],
          })
        ).rejects.toThrow("Note already consumed");
      });

      await waitFor(() => {
        expect(result.current.error?.message).toBe("Note already consumed");
      });
      expect(result.current.stage).toBe("idle");
      expect(result.current.isLoading).toBe(false);
    });

    it("should handle request creation errors", async () => {
      const noteRecords = createNoteRecords(["invalid-format"]);
      const mockClient = createMockWebClient({
        getInputNotes: vi.fn().mockResolvedValue(noteRecords),
        newConsumeTransactionRequest: vi.fn().mockImplementation(() => {
          throw new Error("Invalid note ID format");
        }),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        sync: vi.fn(),
      });

      const { result } = renderHook(() => useConsume());

      await act(async () => {
        await expect(
          result.current.consume({
            accountId: "0x1",
            noteIds: ["invalid-format"],
          })
        ).rejects.toThrow("Invalid note ID format");
      });
    });

    it("should handle account not found errors", async () => {
      const noteRecords = createNoteRecords(["0xnote1"]);
      const mockClient = createMockWebClient({
        getInputNotes: vi.fn().mockResolvedValue(noteRecords),
        newConsumeTransactionRequest: vi
          .fn()
          .mockReturnValue(createMockTransactionRequest()),
        submitNewTransaction: vi
          .fn()
          .mockRejectedValue(new Error("Account not found")),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        sync: vi.fn(),
      });

      const { result } = renderHook(() => useConsume());

      await act(async () => {
        await expect(
          result.current.consume({
            accountId: "0xnonexistent",
            noteIds: ["0xnote1"],
          })
        ).rejects.toThrow("Account not found");
      });
    });
  });

  describe("reset", () => {
    it("should reset all state", async () => {
      const mockTxId = createMockTransactionId();
      const noteRecords = createNoteRecords(["0xnote1"]);
      const mockClient = createMockWebClient({
        getInputNotes: vi.fn().mockResolvedValue(noteRecords),
        newConsumeTransactionRequest: vi
          .fn()
          .mockReturnValue(createMockTransactionRequest()),
        submitNewTransaction: vi.fn().mockResolvedValue(mockTxId),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        sync: vi.fn().mockResolvedValue(undefined),
      });

      const { result } = renderHook(() => useConsume());

      // Execute consume
      await act(async () => {
        await result.current.consume({
          accountId: "0x1",
          noteIds: ["0xnote1"],
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

    it("should allow consuming new notes after reset", async () => {
      const mockTxId1 = createMockTransactionId("0xtx1");
      const mockTxId2 = createMockTransactionId("0xtx2");
      const noteRecordsFirst = createNoteRecords(["0xnote1"]);
      const noteRecordsSecond = createNoteRecords(["0xnote2"]);
      const mockClient = createMockWebClient({
        getInputNotes: vi
          .fn()
          .mockResolvedValueOnce(noteRecordsFirst)
          .mockResolvedValueOnce(noteRecordsSecond),
        newConsumeTransactionRequest: vi
          .fn()
          .mockReturnValue(createMockTransactionRequest()),
        submitNewTransaction: vi
          .fn()
          .mockResolvedValueOnce(mockTxId1)
          .mockResolvedValueOnce(mockTxId2),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        sync: vi.fn().mockResolvedValue(undefined),
      });

      const { result } = renderHook(() => useConsume());

      // First consume
      await act(async () => {
        await result.current.consume({
          accountId: "0x1",
          noteIds: ["0xnote1"],
        });
      });

      expect(result.current.result?.transactionId).toBe("0xtx1");

      // Reset
      act(() => {
        result.current.reset();
      });

      // Second consume
      await act(async () => {
        await result.current.consume({
          accountId: "0x1",
          noteIds: ["0xnote2"],
        });
      });

      expect(result.current.result?.transactionId).toBe("0xtx2");
    });
  });

  describe("sync after consume", () => {
    it("should trigger sync after successful consume", async () => {
      const mockTxId = createMockTransactionId();
      const mockSync = vi.fn().mockResolvedValue(undefined);
      const noteRecords = createNoteRecords(["0xnote1"]);
      const mockClient = createMockWebClient({
        getInputNotes: vi.fn().mockResolvedValue(noteRecords),
        newConsumeTransactionRequest: vi
          .fn()
          .mockReturnValue(createMockTransactionRequest()),
        submitNewTransaction: vi.fn().mockResolvedValue(mockTxId),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        sync: mockSync,
      });

      const { result } = renderHook(() => useConsume());

      await act(async () => {
        await result.current.consume({
          accountId: "0x1",
          noteIds: ["0xnote1"],
        });
      });

      expect(mockSync).toHaveBeenCalledTimes(1);
    });
  });
});
