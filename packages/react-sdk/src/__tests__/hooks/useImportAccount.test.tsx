import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, act, waitFor } from "@testing-library/react";
import { useImportAccount } from "../../hooks/useImportAccount";
import { useMiden } from "../../context/MidenProvider";
import { useMidenStore } from "../../store/MidenStore";
import type { AccountFile } from "@miden-sdk/miden-sdk";
import {
  createMockAccount,
  createMockAccountFile,
  createMockAccountId,
  createMockWebClient,
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

describe("useImportAccount", () => {
  describe("initial state", () => {
    it("should return initial state", () => {
      mockUseMiden.mockReturnValue({
        client: null,
        isReady: false,
        sync: vi.fn(),
      });

      const { result } = renderHook(() => useImportAccount());

      expect(result.current.account).toBeNull();
      expect(result.current.isImporting).toBe(false);
      expect(result.current.error).toBeNull();
      expect(typeof result.current.importAccount).toBe("function");
      expect(typeof result.current.reset).toBe("function");
    });
  });

  describe("import account", () => {
    it("should throw error when client is not ready", async () => {
      mockUseMiden.mockReturnValue({
        client: null,
        isReady: false,
        sync: vi.fn(),
      });

      const { result } = renderHook(() => useImportAccount());

      await expect(
        result.current.importAccount({
          type: "id",
          accountId: "0xaccount",
        })
      ).rejects.toThrow("Miden client is not ready");
    });

    it("should import account from file", async () => {
      const mockAccount = createMockAccount();
      const mockAccountFile = createMockAccountFile(mockAccount);
      const mockClient = createMockWebClient({
        importAccountFile: vi.fn().mockResolvedValue("Imported account"),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
      });

      const { result } = renderHook(() => useImportAccount());

      let imported;
      await act(async () => {
        imported = await result.current.importAccount({
          type: "file",
          file: mockAccountFile as unknown as AccountFile,
        });
      });

      expect(imported).toBe(mockAccount);
      expect(result.current.account).toBe(mockAccount);
      expect(mockClient.importAccountFile).toHaveBeenCalledWith(
        mockAccountFile
      );
      expect(mockClient.getAccounts).toHaveBeenCalled();
    });

    it("should ignore already tracked errors on file import", async () => {
      const mockAccount = createMockAccount();
      const mockAccountFile = createMockAccountFile(mockAccount);
      const mockClient = createMockWebClient({
        importAccountFile: vi
          .fn()
          .mockRejectedValue(
            new Error("account with id 0x123 is already being tracked")
          ),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
      });

      const { result } = renderHook(() => useImportAccount());

      let imported;
      await act(async () => {
        imported = await result.current.importAccount({
          type: "file",
          file: mockAccountFile as unknown as AccountFile,
        });
      });

      expect(imported).toBe(mockAccount);
      expect(result.current.account).toBe(mockAccount);
      expect(mockClient.importAccountFile).toHaveBeenCalledWith(
        mockAccountFile
      );
    });

    it("should import account by id", async () => {
      const mockAccount = createMockAccount({
        id: vi.fn(() => createMockAccountId("0ximported")),
      });
      const mockClient = createMockWebClient({
        importAccountById: vi.fn().mockResolvedValue(undefined),
        getAccount: vi.fn().mockResolvedValue(mockAccount),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
      });

      const { result } = renderHook(() => useImportAccount());

      await act(async () => {
        await result.current.importAccount({
          type: "id",
          accountId: "0ximported",
        });
      });

      expect(mockClient.importAccountById).toHaveBeenCalled();
      expect(mockClient.getAccount).toHaveBeenCalled();
      expect(result.current.account).toBe(mockAccount);
    });

    it("should import public account from seed", async () => {
      const mockAccount = createMockAccount();
      const mockClient = createMockWebClient({
        importPublicAccountFromSeed: vi.fn().mockResolvedValue(mockAccount),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
      });

      const { result } = renderHook(() => useImportAccount());

      await act(async () => {
        await result.current.importAccount({
          type: "seed",
          seed: new Uint8Array([1, 2, 3]),
          mutable: true,
          authScheme: 0,
        });
      });

      expect(mockClient.importPublicAccountFromSeed).toHaveBeenCalled();
      expect(result.current.account).toBe(mockAccount);
    });
  });

  describe("error handling", () => {
    it("should surface errors during import", async () => {
      const mockClient = createMockWebClient({
        importAccountById: vi.fn().mockRejectedValue(new Error("Not found")),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
      });

      const { result } = renderHook(() => useImportAccount());

      await act(async () => {
        await expect(
          result.current.importAccount({
            type: "id",
            accountId: "0xmissing",
          })
        ).rejects.toThrow("Not found");
      });

      await waitFor(() => {
        expect(result.current.error?.message).toBe("Not found");
      });
    });
  });

  describe("reset", () => {
    it("should reset all state", async () => {
      const mockAccount = createMockAccount();
      const mockClient = createMockWebClient({
        importPublicAccountFromSeed: vi.fn().mockResolvedValue(mockAccount),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
      });

      const { result } = renderHook(() => useImportAccount());

      await act(async () => {
        await result.current.importAccount({
          type: "seed",
          seed: new Uint8Array([1, 2, 3]),
        });
      });

      expect(result.current.account).toBe(mockAccount);

      act(() => {
        result.current.reset();
      });

      expect(result.current.account).toBeNull();
      expect(result.current.isImporting).toBe(false);
      expect(result.current.error).toBeNull();
    });
  });
});
