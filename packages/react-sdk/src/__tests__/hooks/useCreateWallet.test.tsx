import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, act, waitFor } from "@testing-library/react";
import { useCreateWallet } from "../../hooks/useCreateWallet";
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

describe("useCreateWallet", () => {
  describe("initial state", () => {
    it("should return initial state", () => {
      mockUseMiden.mockReturnValue({
        client: null,
        isReady: false,
      });

      const { result } = renderHook(() => useCreateWallet());

      expect(result.current.wallet).toBeNull();
      expect(result.current.isCreating).toBe(false);
      expect(result.current.error).toBeNull();
      expect(typeof result.current.createWallet).toBe("function");
      expect(typeof result.current.reset).toBe("function");
    });
  });

  describe("createWallet", () => {
    it("should throw error when client is not ready", async () => {
      mockUseMiden.mockReturnValue({
        client: null,
        isReady: false,
      });

      const { result } = renderHook(() => useCreateWallet());

      await expect(result.current.createWallet()).rejects.toThrow(
        "Miden client is not ready"
      );
    });

    it("should create wallet with default options", async () => {
      const mockWallet = createMockAccount();
      const mockClient = createMockWebClient({
        newWallet: vi.fn().mockResolvedValue(mockWallet),
        getAccounts: vi.fn().mockResolvedValue([createMockAccountHeader()]),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
      });

      const { result } = renderHook(() => useCreateWallet());

      let createdWallet;
      await act(async () => {
        createdWallet = await result.current.createWallet();
      });

      expect(createdWallet).toBe(mockWallet);
      expect(result.current.wallet).toBe(mockWallet);
      expect(result.current.isCreating).toBe(false);
      expect(result.current.error).toBeNull();

      // Verify default options were used
      expect(mockClient.newWallet).toHaveBeenCalledWith(
        expect.anything(), // storageMode.private()
        true, // mutable (default)
        0, // authScheme (default: Falcon)
        undefined // initSeed
      );
    });

    it("should create wallet with custom options", async () => {
      const mockWallet = createMockAccount();
      const mockClient = createMockWebClient({
        newWallet: vi.fn().mockResolvedValue(mockWallet),
        getAccounts: vi.fn().mockResolvedValue([]),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
      });

      const { result } = renderHook(() => useCreateWallet());

      const initSeed = new Uint8Array(32);
      await act(async () => {
        await result.current.createWallet({
          storageMode: "public",
          mutable: false,
          authScheme: 1,
          initSeed,
        });
      });

      expect(mockClient.newWallet).toHaveBeenCalledWith(
        expect.anything(), // storageMode.public()
        false,
        1,
        initSeed
      );
    });

    it("should create wallet with private storage mode", async () => {
      const mockWallet = createMockAccount();
      const mockClient = createMockWebClient({
        newWallet: vi.fn().mockResolvedValue(mockWallet),
        getAccounts: vi.fn().mockResolvedValue([]),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
      });

      const { result } = renderHook(() => useCreateWallet());

      await act(async () => {
        await result.current.createWallet({ storageMode: "private" });
      });

      expect(mockClient.newWallet).toHaveBeenCalled();
    });

    it("should create wallet with network storage mode", async () => {
      const mockWallet = createMockAccount();
      const mockClient = createMockWebClient({
        newWallet: vi.fn().mockResolvedValue(mockWallet),
        getAccounts: vi.fn().mockResolvedValue([]),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
      });

      const { result } = renderHook(() => useCreateWallet());

      await act(async () => {
        await result.current.createWallet({ storageMode: "network" });
      });

      expect(mockClient.newWallet).toHaveBeenCalled();
    });

    it("should refresh accounts list after creation", async () => {
      const mockWallet = createMockAccount();
      const mockAccounts = [createMockAccountHeader()];
      const mockClient = createMockWebClient({
        newWallet: vi.fn().mockResolvedValue(mockWallet),
        getAccounts: vi.fn().mockResolvedValue(mockAccounts),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
      });

      const { result } = renderHook(() => useCreateWallet());

      await act(async () => {
        await result.current.createWallet();
      });

      expect(mockClient.getAccounts).toHaveBeenCalled();
      expect(useMidenStore.getState().accounts).toEqual(mockAccounts);
    });
  });

  describe("loading state", () => {
    it("should set isCreating during wallet creation", async () => {
      let resolvePromise: (value: any) => void;
      const createPromise = new Promise((resolve) => {
        resolvePromise = resolve;
      });

      const mockClient = createMockWebClient({
        newWallet: vi.fn().mockReturnValue(createPromise),
        getAccounts: vi.fn().mockResolvedValue([]),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
      });

      const { result } = renderHook(() => useCreateWallet());

      // Start creation
      let createPromiseRef: Promise<any>;
      act(() => {
        createPromiseRef = result.current.createWallet();
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
      const creationError = new Error("Wallet creation failed");
      const mockClient = createMockWebClient({
        newWallet: vi.fn().mockRejectedValue(creationError),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
      });

      const { result } = renderHook(() => useCreateWallet());

      await act(async () => {
        await expect(result.current.createWallet()).rejects.toThrow(
          "Wallet creation failed"
        );
      });

      await waitFor(() => {
        expect(result.current.error).toBe(creationError);
      });
      expect(result.current.isCreating).toBe(false);
      expect(result.current.wallet).toBeNull();
    });

    it("should convert non-Error objects to Error", async () => {
      const mockClient = createMockWebClient({
        newWallet: vi.fn().mockRejectedValue("String error"),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
      });

      const { result } = renderHook(() => useCreateWallet());

      await act(async () => {
        await expect(result.current.createWallet()).rejects.toThrow();
      });

      await waitFor(() => {
        expect(result.current.error).toBeInstanceOf(Error);
        expect(result.current.error?.message).toBe("String error");
      });
    });
  });

  describe("reset", () => {
    it("should reset all state", async () => {
      const mockWallet = createMockAccount();
      const mockClient = createMockWebClient({
        newWallet: vi.fn().mockResolvedValue(mockWallet),
        getAccounts: vi.fn().mockResolvedValue([]),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
      });

      const { result } = renderHook(() => useCreateWallet());

      // Create wallet
      await act(async () => {
        await result.current.createWallet();
      });

      expect(result.current.wallet).not.toBeNull();

      // Reset
      act(() => {
        result.current.reset();
      });

      expect(result.current.wallet).toBeNull();
      expect(result.current.isCreating).toBe(false);
      expect(result.current.error).toBeNull();
    });

    it("should allow creating new wallet after reset", async () => {
      const mockWallet1 = createMockAccount();
      const mockWallet2 = createMockAccount();
      const mockClient = createMockWebClient({
        newWallet: vi
          .fn()
          .mockResolvedValueOnce(mockWallet1)
          .mockResolvedValueOnce(mockWallet2),
        getAccounts: vi.fn().mockResolvedValue([]),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
      });

      const { result } = renderHook(() => useCreateWallet());

      // First creation
      await act(async () => {
        await result.current.createWallet();
      });

      expect(result.current.wallet).toBe(mockWallet1);

      // Reset
      act(() => {
        result.current.reset();
      });

      // Second creation
      await act(async () => {
        await result.current.createWallet();
      });

      expect(result.current.wallet).toBe(mockWallet2);
    });
  });
});
