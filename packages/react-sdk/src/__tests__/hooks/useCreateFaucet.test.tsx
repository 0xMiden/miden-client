import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, act, waitFor } from "@testing-library/react";
import { useCreateFaucet } from "../../hooks/useCreateFaucet";
import { useMiden } from "../../context/MidenProvider";
import { useMidenStore } from "../../store/MidenStore";
import {
  createMockWebClient,
  createMockAccount,
  createMockAccountHeader,
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

describe("useCreateFaucet", () => {
  describe("initial state", () => {
    it("should return initial state", () => {
      mockUseMiden.mockReturnValue({
        client: null,
        isReady: false,
      });

      const { result } = renderHook(() => useCreateFaucet());

      expect(result.current.faucet).toBeNull();
      expect(result.current.isCreating).toBe(false);
      expect(result.current.error).toBeNull();
      expect(typeof result.current.createFaucet).toBe("function");
      expect(typeof result.current.reset).toBe("function");
    });
  });

  describe("createFaucet", () => {
    it("should throw error when client is not ready", async () => {
      mockUseMiden.mockReturnValue({
        client: null,
        isReady: false,
      });

      const { result } = renderHook(() => useCreateFaucet());

      await expect(
        result.current.createFaucet({
          tokenSymbol: "TEST",
          maxSupply: 1000000n,
        })
      ).rejects.toThrow("Miden client is not ready");
    });

    it("should create faucet with required options", async () => {
      const mockFaucet = createMockAccount({ isFaucet: vi.fn(() => true) });
      const mockClient = createMockWebClient({
        newFaucet: vi.fn().mockResolvedValue(mockFaucet),
        getAccounts: vi.fn().mockResolvedValue([]),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
      });

      const { result } = renderHook(() => useCreateFaucet());

      let createdFaucet;
      await act(async () => {
        createdFaucet = await result.current.createFaucet({
          tokenSymbol: "TEST",
          maxSupply: 1000000n,
        });
      });

      expect(createdFaucet).toBe(mockFaucet);
      expect(result.current.faucet).toBe(mockFaucet);
      expect(result.current.isCreating).toBe(false);
      expect(result.current.error).toBeNull();

      // Verify default options were used
      expect(mockClient.newFaucet).toHaveBeenCalledWith(
        expect.anything(), // storageMode.private() (default)
        false, // nonFungible (always false for now)
        "TEST",
        8, // decimals (default)
        1000000n,
        0 // authScheme (default: Falcon)
      );
    });

    it("should create faucet with custom options", async () => {
      const mockFaucet = createMockAccount({ isFaucet: vi.fn(() => true) });
      const mockClient = createMockWebClient({
        newFaucet: vi.fn().mockResolvedValue(mockFaucet),
        getAccounts: vi.fn().mockResolvedValue([]),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
      });

      const { result } = renderHook(() => useCreateFaucet());

      await act(async () => {
        await result.current.createFaucet({
          tokenSymbol: "USDC",
          maxSupply: 10000000000n,
          decimals: 6,
          storageMode: "public",
          authScheme: 1,
        });
      });

      expect(mockClient.newFaucet).toHaveBeenCalledWith(
        expect.anything(), // storageMode.public()
        false,
        "USDC",
        6,
        10000000000n,
        1
      );
    });

    it("should create faucet with different storage modes", async () => {
      const mockFaucet = createMockAccount();
      const mockClient = createMockWebClient({
        newFaucet: vi.fn().mockResolvedValue(mockFaucet),
        getAccounts: vi.fn().mockResolvedValue([]),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
      });

      const { result } = renderHook(() => useCreateFaucet());

      // Test private
      await act(async () => {
        await result.current.createFaucet({
          tokenSymbol: "A",
          maxSupply: 100n,
          storageMode: "private",
        });
      });

      // Test public
      await act(async () => {
        await result.current.createFaucet({
          tokenSymbol: "B",
          maxSupply: 100n,
          storageMode: "public",
        });
      });

      // Test network
      await act(async () => {
        await result.current.createFaucet({
          tokenSymbol: "C",
          maxSupply: 100n,
          storageMode: "network",
        });
      });

      expect(mockClient.newFaucet).toHaveBeenCalledTimes(3);
    });

    it("should refresh accounts list after creation", async () => {
      const mockFaucet = createMockAccount();
      const mockAccounts = [createMockAccountHeader()];
      const mockClient = createMockWebClient({
        newFaucet: vi.fn().mockResolvedValue(mockFaucet),
        getAccounts: vi.fn().mockResolvedValue(mockAccounts),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
      });

      const { result } = renderHook(() => useCreateFaucet());

      await act(async () => {
        await result.current.createFaucet({
          tokenSymbol: "TEST",
          maxSupply: 1000n,
        });
      });

      expect(mockClient.getAccounts).toHaveBeenCalled();
      expect(useMidenStore.getState().accounts).toEqual(mockAccounts);
    });
  });

  describe("loading state", () => {
    it("should set isCreating during faucet creation", async () => {
      let resolvePromise: (value: any) => void;
      const createPromise = new Promise((resolve) => {
        resolvePromise = resolve;
      });

      const mockClient = createMockWebClient({
        newFaucet: vi.fn().mockReturnValue(createPromise),
        getAccounts: vi.fn().mockResolvedValue([]),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
      });

      const { result } = renderHook(() => useCreateFaucet());

      // Start creation
      let createPromiseRef: Promise<any>;
      act(() => {
        createPromiseRef = result.current.createFaucet({
          tokenSymbol: "TEST",
          maxSupply: 1000n,
        });
      });

      // Should be creating
      await waitFor(() => {
        expect(result.current.isCreating).toBe(true);
      });

      // Resolve
      await act(async () => {
        resolvePromise!(createMockAccount());
        await createPromiseRef;
      });

      await waitFor(() => {
        expect(result.current.isCreating).toBe(false);
      });
    });
  });

  describe("error handling", () => {
    it("should capture and expose creation errors", async () => {
      const creationError = new Error("Faucet creation failed");
      const mockClient = createMockWebClient({
        newFaucet: vi.fn().mockRejectedValue(creationError),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
      });

      const { result } = renderHook(() => useCreateFaucet());

      await act(async () => {
        await expect(
          result.current.createFaucet({
            tokenSymbol: "TEST",
            maxSupply: 1000n,
          })
        ).rejects.toThrow("Faucet creation failed");
      });

      await waitFor(() => {
        expect(result.current.error).toBe(creationError);
      });
      expect(result.current.isCreating).toBe(false);
      expect(result.current.faucet).toBeNull();
    });

    it("should convert non-Error objects to Error", async () => {
      const mockClient = createMockWebClient({
        newFaucet: vi.fn().mockRejectedValue("String error"),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
      });

      const { result } = renderHook(() => useCreateFaucet());

      await act(async () => {
        await expect(
          result.current.createFaucet({
            tokenSymbol: "TEST",
            maxSupply: 1000n,
          })
        ).rejects.toThrow();
      });

      await waitFor(() => {
        expect(result.current.error).toBeInstanceOf(Error);
      });
    });
  });

  describe("reset", () => {
    it("should reset all state", async () => {
      const mockFaucet = createMockAccount();
      const mockClient = createMockWebClient({
        newFaucet: vi.fn().mockResolvedValue(mockFaucet),
        getAccounts: vi.fn().mockResolvedValue([]),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
      });

      const { result } = renderHook(() => useCreateFaucet());

      // Create faucet
      await act(async () => {
        await result.current.createFaucet({
          tokenSymbol: "TEST",
          maxSupply: 1000n,
        });
      });

      expect(result.current.faucet).not.toBeNull();

      // Reset
      act(() => {
        result.current.reset();
      });

      expect(result.current.faucet).toBeNull();
      expect(result.current.isCreating).toBe(false);
      expect(result.current.error).toBeNull();
    });
  });

  describe("bigint handling", () => {
    it("should handle large maxSupply values", async () => {
      const mockFaucet = createMockAccount();
      const mockClient = createMockWebClient({
        newFaucet: vi.fn().mockResolvedValue(mockFaucet),
        getAccounts: vi.fn().mockResolvedValue([]),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
      });

      const { result } = renderHook(() => useCreateFaucet());

      const largeSupply = 1000000000000000000n; // 1 quintillion

      await act(async () => {
        await result.current.createFaucet({
          tokenSymbol: "BIG",
          maxSupply: largeSupply,
        });
      });

      expect(mockClient.newFaucet).toHaveBeenCalledWith(
        expect.anything(),
        false,
        "BIG",
        8,
        largeSupply,
        0
      );
    });
  });
});
