import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, act, waitFor } from "@testing-library/react";
import { useExportNote } from "../../hooks/useExportNote";
import { useMiden } from "../../context/MidenProvider";
import { createMockWebClient } from "../mocks/miden-sdk";

vi.mock("../../context/MidenProvider", () => ({
  useMiden: vi.fn(),
}));

const mockUseMiden = useMiden as ReturnType<typeof vi.fn>;

beforeEach(() => {
  vi.clearAllMocks();
});

describe("useExportNote", () => {
  describe("initial state", () => {
    it("should return initial state", () => {
      mockUseMiden.mockReturnValue({
        client: null,
        isReady: false,
        runExclusive: vi.fn(),
      });

      const { result } = renderHook(() => useExportNote());

      expect(result.current.isExporting).toBe(false);
      expect(result.current.error).toBeNull();
      expect(typeof result.current.exportNote).toBe("function");
      expect(typeof result.current.reset).toBe("function");
    });
  });

  describe("exportNote", () => {
    it("should throw when client is not ready", async () => {
      mockUseMiden.mockReturnValue({
        client: null,
        isReady: false,
        runExclusive: vi.fn(),
      });

      const { result } = renderHook(() => useExportNote());

      await expect(result.current.exportNote("0xnote1")).rejects.toThrow(
        "Miden client is not ready"
      );
    });

    it("should export note and return serialized bytes", async () => {
      const serializedBytes = new Uint8Array([10, 20, 30]);
      const mockNoteFile = { serialize: vi.fn(() => serializedBytes) };
      const mockClient = createMockWebClient({
        exportNoteFile: vi.fn().mockResolvedValue(mockNoteFile),
      });
      const runExclusive = vi.fn((fn: () => unknown) => fn());

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        runExclusive,
      });

      const { result } = renderHook(() => useExportNote());

      let bytes: Uint8Array;
      await act(async () => {
        bytes = await result.current.exportNote("0xnote1");
      });

      expect(bytes!).toBe(serializedBytes);
      expect(mockClient.exportNoteFile).toHaveBeenCalledWith(
        "0xnote1",
        expect.anything() // NoteExportFormat.Full
      );
      expect(mockNoteFile.serialize).toHaveBeenCalled();
      expect(result.current.isExporting).toBe(false);
    });

    it("should set error on failure", async () => {
      const mockClient = createMockWebClient({
        exportNoteFile: vi.fn().mockRejectedValue(new Error("Note not found")),
      });
      const runExclusive = vi.fn((fn: () => unknown) => fn());

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        runExclusive,
      });

      const { result } = renderHook(() => useExportNote());

      await act(async () => {
        await expect(result.current.exportNote("0xbadnote")).rejects.toThrow(
          "Note not found"
        );
      });

      await waitFor(() => {
        expect(result.current.error?.message).toBe("Note not found");
      });
      expect(result.current.isExporting).toBe(false);
    });
  });

  describe("reset", () => {
    it("should reset error state", async () => {
      const mockClient = createMockWebClient({
        exportNoteFile: vi.fn().mockRejectedValue(new Error("fail")),
      });
      const runExclusive = vi.fn((fn: () => unknown) => fn());

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        runExclusive,
      });

      const { result } = renderHook(() => useExportNote());

      await act(async () => {
        await result.current.exportNote("0x1").catch(() => {});
      });

      expect(result.current.error).not.toBeNull();

      act(() => {
        result.current.reset();
      });

      expect(result.current.error).toBeNull();
      expect(result.current.isExporting).toBe(false);
    });
  });
});
