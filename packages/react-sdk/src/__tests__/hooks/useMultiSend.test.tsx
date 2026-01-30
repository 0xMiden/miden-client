import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useMultiSend } from "../../hooks/useMultiSend";
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

describe("useMultiSend", () => {
  describe("initial state", () => {
    it("should return initial state", () => {
      mockUseMiden.mockReturnValue({
        client: null,
        isReady: false,
        sync: vi.fn(),
      });

      const { result } = renderHook(() => useMultiSend());

      expect(result.current.result).toBeNull();
      expect(result.current.isLoading).toBe(false);
      expect(result.current.stage).toBe("idle");
      expect(result.current.error).toBeNull();
      expect(typeof result.current.sendMany).toBe("function");
      expect(typeof result.current.reset).toBe("function");
    });
  });

  describe("multi-send transaction", () => {
    it("should throw error when client is not ready", async () => {
      mockUseMiden.mockReturnValue({
        client: null,
        isReady: false,
        sync: vi.fn(),
      });

      const { result } = renderHook(() => useMultiSend());

      await expect(
        result.current.sendMany({
          from: "0xsender",
          assetId: "0xfaucet",
          recipients: [{ to: "0xrecipient", amount: 100n }],
        })
      ).rejects.toThrow("Miden client is not ready");
    });

    it("should throw error when no recipients are provided", async () => {
      const mockClient = createMockWebClient();
      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        sync: vi.fn(),
      });

      const { result } = renderHook(() => useMultiSend());

      await expect(
        result.current.sendMany({
          from: "0xsender",
          assetId: "0xfaucet",
          recipients: [],
        })
      ).rejects.toThrow("No recipients provided");
    });

    it("should execute multi-send with default note type", async () => {
      const mockTxId = createMockTransactionId("0xmultisend");
      const mockSync = vi.fn().mockResolvedValue(undefined);
      const mockClient = createMockWebClient({
        submitNewTransaction: vi.fn().mockResolvedValue(mockTxId),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        sync: mockSync,
      });

      const { result } = renderHook(() => useMultiSend());

      let txResult;
      await act(async () => {
        txResult = await result.current.sendMany({
          from: "0xsender",
          assetId: "0xfaucet",
          recipients: [
            { to: "0xrecipient1", amount: 100n },
            { to: "0xrecipient2", amount: 200n },
          ],
        });
      });

      expect(txResult).toEqual({ transactionId: "0xmultisend" });
      expect(result.current.result).toEqual({ transactionId: "0xmultisend" });
      expect(result.current.stage).toBe("complete");
      expect(mockSync).toHaveBeenCalled();

      const createP2IDNoteMock = (
        Note as unknown as { createP2IDNote: ReturnType<typeof vi.fn> }
      ).createP2IDNote;

      expect(createP2IDNoteMock).toHaveBeenCalledTimes(2);
      expect(createP2IDNoteMock).toHaveBeenCalledWith(
        expect.anything(),
        expect.anything(),
        expect.anything(),
        NoteType.Private,
        expect.anything()
      );
    });

    it("should execute multi-send with custom note type", async () => {
      const mockTxId = createMockTransactionId();
      const mockClient = createMockWebClient({
        submitNewTransaction: vi.fn().mockResolvedValue(mockTxId),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        sync: vi.fn(),
      });

      const { result } = renderHook(() => useMultiSend());

      await act(async () => {
        await result.current.sendMany({
          from: "0xsender",
          assetId: "0xfaucet",
          recipients: [{ to: "0xrecipient", amount: 100n }],
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
});
