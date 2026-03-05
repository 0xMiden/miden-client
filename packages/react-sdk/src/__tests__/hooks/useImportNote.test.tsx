import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, act, waitFor } from "@testing-library/react";
import { useImportNote } from "../../hooks/useImportNote";
import { useMiden } from "../../context/MidenProvider";
import { NoteFile } from "@miden-sdk/miden-sdk";
import { createMockWebClient } from "../mocks/miden-sdk";

vi.mock("../../context/MidenProvider", () => ({
  useMiden: vi.fn(),
}));

const mockUseMiden = useMiden as ReturnType<typeof vi.fn>;

beforeEach(() => {
  vi.clearAllMocks();
});

describe("useImportNote", () => {
  describe("initial state", () => {
    it("should return initial state", () => {
      mockUseMiden.mockReturnValue({
        client: null,
        isReady: false,
        runExclusive: vi.fn(),
        sync: vi.fn(),
      });

      const { result } = renderHook(() => useImportNote());

      expect(result.current.isImporting).toBe(false);
      expect(result.current.error).toBeNull();
      expect(typeof result.current.importNote).toBe("function");
      expect(typeof result.current.reset).toBe("function");
    });
  });

  describe("importNote", () => {
    it("should throw when client is not ready", async () => {
      mockUseMiden.mockReturnValue({
        client: null,
        isReady: false,
        runExclusive: vi.fn(),
        sync: vi.fn(),
      });

      const { result } = renderHook(() => useImportNote());

      await expect(
        result.current.importNote(new Uint8Array([1, 2, 3]))
      ).rejects.toThrow("Miden client is not ready");
    });

    it("should import note and return noteId", async () => {
      const mockNoteId = { toString: () => "0xnote_abc" };
      const mockSync = vi.fn().mockResolvedValue(undefined);
      const mockClient = createMockWebClient({
        importNoteFile: vi.fn().mockResolvedValue(mockNoteId),
      });
      const runExclusive = vi.fn((fn: () => unknown) => fn());

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        runExclusive,
        sync: mockSync,
      });

      const { result } = renderHook(() => useImportNote());
      const noteBytes = new Uint8Array([1, 2, 3]);

      let noteId: string;
      await act(async () => {
        noteId = await result.current.importNote(noteBytes);
      });

      expect(noteId!).toBe("0xnote_abc");
      expect(
        (NoteFile as unknown as { deserialize: ReturnType<typeof vi.fn> })
          .deserialize
      ).toHaveBeenCalledWith(noteBytes);
      expect(mockClient.importNoteFile).toHaveBeenCalled();
      expect(result.current.isImporting).toBe(false);
    });

    it("should call sync after successful import", async () => {
      const mockSync = vi.fn().mockResolvedValue(undefined);
      const mockClient = createMockWebClient({
        importNoteFile: vi
          .fn()
          .mockResolvedValue({ toString: () => "0xnote" }),
      });
      const runExclusive = vi.fn((fn: () => unknown) => fn());

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        runExclusive,
        sync: mockSync,
      });

      const { result } = renderHook(() => useImportNote());

      await act(async () => {
        await result.current.importNote(new Uint8Array([1]));
      });

      expect(mockSync).toHaveBeenCalled();
    });

    it("should set error on failure", async () => {
      const mockClient = createMockWebClient({
        importNoteFile: vi
          .fn()
          .mockRejectedValue(new Error("Invalid note bytes")),
      });
      const runExclusive = vi.fn((fn: () => unknown) => fn());

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        runExclusive,
        sync: vi.fn(),
      });

      const { result } = renderHook(() => useImportNote());

      await act(async () => {
        await expect(
          result.current.importNote(new Uint8Array([0xff]))
        ).rejects.toThrow("Invalid note bytes");
      });

      await waitFor(() => {
        expect(result.current.error?.message).toBe("Invalid note bytes");
      });
      expect(result.current.isImporting).toBe(false);
    });
  });

  describe("reset", () => {
    it("should reset error state", async () => {
      const mockClient = createMockWebClient({
        importNoteFile: vi.fn().mockRejectedValue(new Error("fail")),
      });
      const runExclusive = vi.fn((fn: () => unknown) => fn());

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        runExclusive,
        sync: vi.fn(),
      });

      const { result } = renderHook(() => useImportNote());

      await act(async () => {
        await result.current.importNote(new Uint8Array([])).catch(() => {});
      });

      expect(result.current.error).not.toBeNull();

      act(() => {
        result.current.reset();
      });

      expect(result.current.error).toBeNull();
      expect(result.current.isImporting).toBe(false);
    });
  });
});
