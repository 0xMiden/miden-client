import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, act, waitFor } from "@testing-library/react";
import { useExportStore } from "../../hooks/useExportStore";
import { useMiden } from "../../context/MidenProvider";
import { createMockWebClient } from "../mocks/miden-sdk";
import { exportStore as sdkExportStore } from "@miden-sdk/miden-sdk";

vi.mock("../../context/MidenProvider", () => ({
  useMiden: vi.fn(),
}));

vi.mock("@miden-sdk/miden-sdk", () => ({
  exportStore: vi.fn(),
}));

const mockUseMiden = useMiden as ReturnType<typeof vi.fn>;
const mockSdkExportStore = sdkExportStore as ReturnType<typeof vi.fn>;

beforeEach(() => {
  vi.clearAllMocks();
});

describe("useExportStore", () => {
  describe("initial state", () => {
    it("should return initial state", () => {
      mockUseMiden.mockReturnValue({
        client: null,
        isReady: false,
        runExclusive: vi.fn(),
      });

      const { result } = renderHook(() => useExportStore());

      expect(result.current.isExporting).toBe(false);
      expect(result.current.error).toBeNull();
      expect(typeof result.current.exportStore).toBe("function");
      expect(typeof result.current.reset).toBe("function");
    });
  });

  describe("exportStore", () => {
    it("should throw when client is not ready", async () => {
      mockUseMiden.mockReturnValue({
        client: null,
        isReady: false,
        runExclusive: vi.fn(),
      });

      const { result } = renderHook(() => useExportStore());

      await expect(result.current.exportStore()).rejects.toThrow(
        "Miden client is not ready"
      );
    });

    it("should export store successfully", async () => {
      const storeData = '{"tables":{}}';
      const mockClient = createMockWebClient({
        storeIdentifier: vi.fn().mockReturnValue("MyStore"),
      });
      const runExclusive = vi.fn((fn: () => unknown) => fn());
      mockSdkExportStore.mockResolvedValue(storeData);

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        runExclusive,
      });

      const { result } = renderHook(() => useExportStore());

      let exportResult: unknown;
      await act(async () => {
        exportResult = await result.current.exportStore();
      });

      expect(exportResult).toBe(storeData);
      expect(mockSdkExportStore).toHaveBeenCalledWith("MyStore");
      expect(result.current.isExporting).toBe(false);
      expect(result.current.error).toBeNull();
    });

    it("should set error on failure", async () => {
      const mockClient = createMockWebClient({
        storeIdentifier: vi.fn().mockReturnValue("MyStore"),
      });
      const runExclusive = vi.fn((fn: () => unknown) => fn());
      mockSdkExportStore.mockRejectedValue(new Error("Export failed"));

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        runExclusive,
      });

      const { result } = renderHook(() => useExportStore());

      await act(async () => {
        await expect(result.current.exportStore()).rejects.toThrow(
          "Export failed"
        );
      });

      await waitFor(() => {
        expect(result.current.error?.message).toBe("Export failed");
      });
      expect(result.current.isExporting).toBe(false);
    });

    it("should set isExporting during operation", async () => {
      let resolveExport: (value: string) => void;
      const exportPromise = new Promise<string>((resolve) => {
        resolveExport = resolve;
      });

      const mockClient = createMockWebClient({
        storeIdentifier: vi.fn().mockReturnValue("MyStore"),
      });
      const runExclusive = vi.fn((fn: () => unknown) => fn());
      mockSdkExportStore.mockReturnValue(exportPromise);

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        runExclusive,
      });

      const { result } = renderHook(() => useExportStore());

      let promise: Promise<unknown>;
      act(() => {
        promise = result.current.exportStore();
      });

      await waitFor(() => {
        expect(result.current.isExporting).toBe(true);
      });

      await act(async () => {
        resolveExport!("{}");
        await promise;
      });

      expect(result.current.isExporting).toBe(false);
    });
  });

  describe("reset", () => {
    it("should reset error state", async () => {
      const mockClient = createMockWebClient({
        storeIdentifier: vi.fn().mockReturnValue("MyStore"),
      });
      const runExclusive = vi.fn((fn: () => unknown) => fn());
      mockSdkExportStore.mockRejectedValue(new Error("fail"));

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        runExclusive,
      });

      const { result } = renderHook(() => useExportStore());

      await act(async () => {
        await result.current.exportStore().catch(() => {});
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
