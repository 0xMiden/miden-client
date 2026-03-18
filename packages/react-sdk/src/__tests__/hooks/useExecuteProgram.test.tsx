import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, act, waitFor } from "@testing-library/react";
import { useExecuteProgram } from "../../hooks/useExecuteProgram";
import { useMiden } from "../../context/MidenProvider";
import { useMidenStore } from "../../store/MidenStore";
import { createMockWebClient, createMockFeltArray } from "../mocks/miden-sdk";

// Mock useMiden
vi.mock("../../context/MidenProvider", () => ({
  useMiden: vi.fn(),
}));

const mockUseMiden = useMiden as ReturnType<typeof vi.fn>;

beforeEach(() => {
  useMidenStore.getState().reset();
  vi.clearAllMocks();
});

describe("useExecuteProgram", () => {
  describe("initial state", () => {
    it("should return initial state", () => {
      mockUseMiden.mockReturnValue({
        client: null,
        isReady: false,
        sync: vi.fn(),
      });

      const { result } = renderHook(() => useExecuteProgram());

      expect(result.current.result).toBeNull();
      expect(result.current.isLoading).toBe(false);
      expect(result.current.error).toBeNull();
      expect(typeof result.current.execute).toBe("function");
      expect(typeof result.current.reset).toBe("function");
    });
  });

  describe("execute program", () => {
    it("should throw error when client is not ready", async () => {
      mockUseMiden.mockReturnValue({
        client: null,
        isReady: false,
        sync: vi.fn(),
      });

      const { result } = renderHook(() => useExecuteProgram());

      await expect(
        result.current.execute({
          accountId: "0xaccount",
          script: {},
        })
      ).rejects.toThrow("Miden client is not ready");
    });

    it("should execute program and return stack as bigint[]", async () => {
      const mockFeltArray = createMockFeltArray(16);
      const mockSync = vi.fn().mockResolvedValue(undefined);
      const mockClient = createMockWebClient({
        executeProgram: vi.fn().mockResolvedValue(mockFeltArray),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        sync: mockSync,
      });

      const { result } = renderHook(() => useExecuteProgram());

      let execResult;
      await act(async () => {
        execResult = await result.current.execute({
          accountId: "0xaccount",
          script: {},
        });
      });

      expect(execResult).toBeDefined();
      expect(execResult!.stack).toHaveLength(16);
      expect(execResult!.stack).toEqual(
        Array.from({ length: 16 }, (_, i) => BigInt(i))
      );
      expect(result.current.result).toEqual(execResult);
      expect(mockClient.executeProgram).toHaveBeenCalledWith(
        expect.anything(),
        {},
        expect.anything(),
        expect.anything()
      );
      expect(mockSync).toHaveBeenCalled();
    });

    it("should pass adviceInputs and foreignAccounts to client", async () => {
      const mockFeltArray = createMockFeltArray(16);
      const mockSync = vi.fn().mockResolvedValue(undefined);
      const mockClient = createMockWebClient({
        executeProgram: vi.fn().mockResolvedValue(mockFeltArray),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        sync: mockSync,
      });

      const { result } = renderHook(() => useExecuteProgram());

      const mockAdviceInputs = {};
      const mockForeignAccounts = ["0xforeign1", "0xforeign2"];

      await act(async () => {
        await result.current.execute({
          accountId: "0xaccount",
          script: {},
          adviceInputs: mockAdviceInputs as any,
          foreignAccounts: mockForeignAccounts,
        });
      });

      expect(mockClient.executeProgram).toHaveBeenCalledWith(
        expect.anything(),
        {},
        mockAdviceInputs,
        expect.anything()
      );
    });
  });

  describe("error handling", () => {
    it("should capture execution errors and set error state", async () => {
      const execError = new Error("Execution failed");
      const mockClient = createMockWebClient({
        executeProgram: vi.fn().mockRejectedValue(execError),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        sync: vi.fn().mockResolvedValue(undefined),
      });

      const { result } = renderHook(() => useExecuteProgram());

      await act(async () => {
        await expect(
          result.current.execute({
            accountId: "0x1",
            script: {},
          })
        ).rejects.toThrow("Execution failed");
      });

      await waitFor(() => {
        expect(result.current.error?.message).toBe("Execution failed");
      });
      expect(result.current.isLoading).toBe(false);
    });
  });

  describe("concurrency guard", () => {
    it("should reject concurrent executions with OPERATION_BUSY", async () => {
      let resolveExecute: () => void;
      const executePromise = new Promise(
        (resolve) => (resolveExecute = () => resolve(createMockFeltArray(16)))
      );

      const mockClient = createMockWebClient({
        executeProgram: vi.fn().mockReturnValue(executePromise),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        sync: vi.fn().mockResolvedValue(undefined),
      });

      const { result } = renderHook(() => useExecuteProgram());

      let firstExec: Promise<any>;
      act(() => {
        firstExec = result.current.execute({
          accountId: "0x1",
          script: {},
        });
      });

      await expect(
        result.current.execute({
          accountId: "0x1",
          script: {},
        })
      ).rejects.toThrow("A program execution is already in progress");

      await act(async () => {
        resolveExecute!();
        await firstExec;
      });
    });
  });

  describe("auto-sync", () => {
    it("should call sync before execute by default", async () => {
      const mockSync = vi.fn().mockResolvedValue(undefined);
      const mockFeltArray = createMockFeltArray(16);
      const mockClient = createMockWebClient({
        executeProgram: vi.fn().mockResolvedValue(mockFeltArray),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        sync: mockSync,
      });

      const { result } = renderHook(() => useExecuteProgram());

      await act(async () => {
        await result.current.execute({
          accountId: "0x1",
          script: {},
        });
      });

      expect(mockSync).toHaveBeenCalled();
    });

    it("should skip pre-sync when skipSync is true", async () => {
      const mockSync = vi.fn().mockResolvedValue(undefined);
      const mockFeltArray = createMockFeltArray(16);
      const mockClient = createMockWebClient({
        executeProgram: vi.fn().mockResolvedValue(mockFeltArray),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        sync: mockSync,
      });

      const { result } = renderHook(() => useExecuteProgram());

      await act(async () => {
        await result.current.execute({
          accountId: "0x1",
          script: {},
          skipSync: true,
        });
      });

      expect(mockSync).not.toHaveBeenCalled();
    });
  });

  describe("reset", () => {
    it("should reset state", async () => {
      const mockFeltArray = createMockFeltArray(16);
      const mockClient = createMockWebClient({
        executeProgram: vi.fn().mockResolvedValue(mockFeltArray),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
        sync: vi.fn().mockResolvedValue(undefined),
      });

      const { result } = renderHook(() => useExecuteProgram());

      await act(async () => {
        await result.current.execute({
          accountId: "0x1",
          script: {},
        });
      });

      expect(result.current.result).not.toBeNull();

      act(() => {
        result.current.reset();
      });

      expect(result.current.result).toBeNull();
      expect(result.current.isLoading).toBe(false);
      expect(result.current.error).toBeNull();
    });
  });
});
