import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useInternalTransfer } from "../../hooks/useInternalTransfer";
import { useMiden } from "../../context/MidenProvider";
import { useMidenStore } from "../../store/MidenStore";
import { Note, NoteType } from "@miden-sdk/miden-sdk";
import {
  createMockWebClient,
  createMockTransactionId,
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

describe("useInternalTransfer", () => {
  describe("initial state", () => {
    it("should return initial state", () => {
      mockUseMiden.mockReturnValue({
        client: null,
        isReady: false,
        sync: vi.fn(),
      });

      const { result } = renderHook(() => useInternalTransfer());

      expect(result.current.result).toBeNull();
      expect(result.current.isLoading).toBe(false);
      expect(result.current.stage).toBe("idle");
      expect(result.current.error).toBeNull();
      expect(typeof result.current.transfer).toBe("function");
      expect(typeof result.current.transferChain).toBe("function");
      expect(typeof result.current.reset).toBe("function");
    });
  });

  describe("transfer", () => {
    it("should throw error when client is not ready", async () => {
      mockUseMiden.mockReturnValue({
        client: null,
        isReady: false,
        sync: vi.fn(),
      });

      const { result } = renderHook(() => useInternalTransfer());

      await expect(
        result.current.transfer({
          from: "0xsender",
          to: "0xrecipient",
          assetId: "0xfaucet",
          amount: 10n,
        })
      ).rejects.toThrow("Miden client is not ready");
    });

    it("should execute transfer with default note type", async () => {
      const createTxId = createMockTransactionId("0xcreate");
      const consumeTxId = createMockTransactionId("0xconsume");
      const mockSync = vi.fn().mockResolvedValue(undefined);
      const mockClient = createMockWebClient({
        submitNewTransaction: vi
          .fn()
          .mockResolvedValueOnce(createTxId)
          .mockResolvedValueOnce(consumeTxId),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        sync: mockSync,
      });

      const { result } = renderHook(() => useInternalTransfer());

      let txResult;
      await act(async () => {
        txResult = await result.current.transfer({
          from: "0xsender",
          to: "0xrecipient",
          assetId: "0xfaucet",
          amount: 25n,
        });
      });

      expect(txResult).toEqual({
        createTransactionId: "0xcreate",
        consumeTransactionId: "0xconsume",
        noteId: expect.any(String),
      });
      expect(result.current.result).toEqual(txResult);
      expect(result.current.stage).toBe("complete");
      expect(mockSync).toHaveBeenCalled();

      const createP2IDNoteMock = (
        Note as unknown as { createP2IDNote: ReturnType<typeof vi.fn> }
      ).createP2IDNote;
      expect(createP2IDNoteMock).toHaveBeenCalledWith(
        expect.anything(),
        expect.anything(),
        expect.anything(),
        NoteType.Private,
        expect.anything()
      );
    });

    it("should execute transfer with custom note type", async () => {
      const createTxId = createMockTransactionId();
      const consumeTxId = createMockTransactionId();
      const mockClient = createMockWebClient({
        submitNewTransaction: vi
          .fn()
          .mockResolvedValueOnce(createTxId)
          .mockResolvedValueOnce(consumeTxId),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        sync: vi.fn(),
      });

      const { result } = renderHook(() => useInternalTransfer());

      await act(async () => {
        await result.current.transfer({
          from: "0xsender",
          to: "0xrecipient",
          assetId: "0xfaucet",
          amount: 25n,
          noteType: "public",
        });
      });

      const createP2IDNoteMock = (
        Note as unknown as { createP2IDNote: ReturnType<typeof vi.fn> }
      ).createP2IDNote;
      expect(createP2IDNoteMock).toHaveBeenCalledWith(
        expect.anything(),
        expect.anything(),
        expect.anything(),
        NoteType.Public,
        expect.anything()
      );
    });
  });

  describe("transferChain", () => {
    it("should throw error when no recipients are provided", async () => {
      const mockClient = createMockWebClient();
      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        sync: vi.fn(),
      });

      const { result } = renderHook(() => useInternalTransfer());

      await expect(
        result.current.transferChain({
          from: "0xsender",
          recipients: [],
          assetId: "0xfaucet",
          amount: 10n,
        })
      ).rejects.toThrow("No recipients provided");
    });

    it("should execute transfer chain across recipients", async () => {
      const mockSync = vi.fn().mockResolvedValue(undefined);
      const mockClient = createMockWebClient({
        submitNewTransaction: vi
          .fn()
          .mockResolvedValue(createMockTransactionId("0xtx")),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        sync: mockSync,
      });

      const { result } = renderHook(() => useInternalTransfer());

      let chainResult;
      await act(async () => {
        chainResult = await result.current.transferChain({
          from: "0xsender",
          recipients: ["0xone", "0xtwo"],
          assetId: "0xfaucet",
          amount: 5n,
        });
      });

      expect(chainResult).toHaveLength(2);
      expect(result.current.stage).toBe("complete");
      expect(mockSync).toHaveBeenCalled();
    });
  });
});
