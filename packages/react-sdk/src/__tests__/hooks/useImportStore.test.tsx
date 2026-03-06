import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, act, waitFor } from "@testing-library/react";
import { useImportStore } from "../../hooks/useImportStore";
import { useMiden } from "../../context/MidenProvider";
import { createMockWebClient } from "../mocks/miden-sdk";

vi.mock("../../context/MidenProvider", () => ({
  useMiden: vi.fn(),
}));

const mockUseMiden = useMiden as ReturnType<typeof vi.fn>;

beforeEach(() => {
  vi.clearAllMocks();
});

describe("useImportStore", () => {
  describe("initial state", () => {
    it("should return initial state", () => {
      mockUseMiden.mockReturnValue({
        client: null,
        isReady: false,
        runExclusive: vi.fn(),
        sync: vi.fn(),
      });

      const { result } = renderHook(() => useImportStore());

      expect(result.current.isImporting).toBe(false);
      expect(result.current.error).toBeNull();
      expect(typeof result.current.importStore).toBe("function");
      expect(typeof result.current.reset).toBe("function");
    });
  });

  describe("importStore", () => {
    it("should throw when client is not ready", async () => {
      mockUseMiden.mockReturnValue({
        client: null,
        isReady: false,
        runExclusive: vi.fn(),
        sync: vi.fn(),
      });

      const { result } = renderHook(() => useImportStore());

      await expect(result.current.importStore({}, "TestStore")).rejects.toThrow(
        "Miden client is not ready"
      );
    });

    it("should import store successfully", async () => {
      const mockSync = vi.fn().mockResolvedValue(undefined);
      const mockClient = createMockWebClient({
        forceImportStore: vi.fn().mockResolvedValue(undefined),
      });
      const runExclusive = vi.fn((fn: () => unknown) => fn());

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        runExclusive,
        sync: mockSync,
      });

      const { result } = renderHook(() => useImportStore());
      const dump = { tables: { accounts: [] } };

      await act(async () => {
        await result.current.importStore(dump, "RestoredStore");
      });

      expect(mockClient.forceImportStore).toHaveBeenCalledWith(
        dump,
        "RestoredStore"
      );
      expect(result.current.isImporting).toBe(false);
      expect(result.current.error).toBeNull();
    });

    it("should call sync after successful import", async () => {
      const mockSync = vi.fn().mockResolvedValue(undefined);
      const mockClient = createMockWebClient({
        forceImportStore: vi.fn().mockResolvedValue(undefined),
      });
      const runExclusive = vi.fn((fn: () => unknown) => fn());

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        runExclusive,
        sync: mockSync,
      });

      const { result } = renderHook(() => useImportStore());

      await act(async () => {
        await result.current.importStore({}, "Store");
      });

      expect(mockSync).toHaveBeenCalled();
    });

    it("should set error on failure", async () => {
      const mockClient = createMockWebClient({
        forceImportStore: vi.fn().mockRejectedValue(new Error("Import failed")),
      });
      const runExclusive = vi.fn((fn: () => unknown) => fn());

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        runExclusive,
        sync: vi.fn(),
      });

      const { result } = renderHook(() => useImportStore());

      await act(async () => {
        await expect(result.current.importStore({}, "Store")).rejects.toThrow(
          "Import failed"
        );
      });

      await waitFor(() => {
        expect(result.current.error?.message).toBe("Import failed");
      });
      expect(result.current.isImporting).toBe(false);
    });

    it("should not call sync on failure", async () => {
      const mockSync = vi.fn();
      const mockClient = createMockWebClient({
        forceImportStore: vi.fn().mockRejectedValue(new Error("Import failed")),
      });
      const runExclusive = vi.fn((fn: () => unknown) => fn());

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        runExclusive,
        sync: mockSync,
      });

      const { result } = renderHook(() => useImportStore());

      await act(async () => {
        await result.current.importStore({}, "Store").catch(() => {});
      });

      expect(mockSync).not.toHaveBeenCalled();
    });
  });

  describe("reset", () => {
    it("should reset error state", async () => {
      const mockClient = createMockWebClient({
        forceImportStore: vi.fn().mockRejectedValue(new Error("fail")),
      });
      const runExclusive = vi.fn((fn: () => unknown) => fn());

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        runExclusive,
        sync: vi.fn(),
      });

      const { result } = renderHook(() => useImportStore());

      await act(async () => {
        await result.current.importStore({}, "S").catch(() => {});
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
