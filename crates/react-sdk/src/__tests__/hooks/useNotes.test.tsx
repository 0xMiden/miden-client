import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, act, waitFor } from "@testing-library/react";
import { useNotes } from "../../hooks/useNotes";
import { useMiden } from "../../context/MidenProvider";
import { useMidenStore } from "../../store/MidenStore";
import {
  createMockWebClient,
  createMockInputNoteRecord,
  createMockConsumableNoteRecord,
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

describe("useNotes", () => {
  describe("initial state", () => {
    it("should return empty arrays when client is not ready", () => {
      mockUseMiden.mockReturnValue({
        client: null,
        isReady: false,
      });

      const { result } = renderHook(() => useNotes());

      expect(result.current.notes).toEqual([]);
      expect(result.current.consumableNotes).toEqual([]);
      expect(result.current.isLoading).toBe(false);
      expect(result.current.error).toBeNull();
    });
  });

  describe("fetching notes", () => {
    it("should fetch notes on mount when client is ready", async () => {
      const mockNotes = [
        createMockInputNoteRecord("0xnote1"),
        createMockInputNoteRecord("0xnote2"),
      ];
      const mockConsumable = [createMockConsumableNoteRecord("0xnote1")];

      const mockClient = createMockWebClient({
        getInputNotes: vi.fn().mockResolvedValue(mockNotes),
        getConsumableNotes: vi.fn().mockResolvedValue(mockConsumable),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
      });

      act(() => {
        useMidenStore.getState().setClient(mockClient as any);
      });

      const { result } = renderHook(() => useNotes());

      await waitFor(() => {
        expect(result.current.notes.length).toBeGreaterThan(0);
      });

      expect(mockClient.getInputNotes).toHaveBeenCalled();
      expect(mockClient.getConsumableNotes).toHaveBeenCalled();
    });

    it("should refetch notes after sync updates", async () => {
      const mockNotes = [createMockInputNoteRecord("0xnote1")];
      const mockConsumable = [createMockConsumableNoteRecord("0xnote1")];

      const mockClient = createMockWebClient({
        getInputNotes: vi.fn().mockResolvedValue(mockNotes),
        getConsumableNotes: vi.fn().mockResolvedValue(mockConsumable),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
      });

      act(() => {
        useMidenStore.getState().setClient(mockClient as any);
      });

      const { result } = renderHook(() => useNotes());

      await waitFor(() => {
        expect(result.current.notes.length).toBeGreaterThan(0);
      });

      expect(mockClient.getInputNotes).toHaveBeenCalledTimes(1);

      act(() => {
        useMidenStore.getState().setSyncState({ lastSyncTime: Date.now() });
      });

      await waitFor(() => {
        expect(mockClient.getInputNotes).toHaveBeenCalledTimes(2);
      });
    });

    it("should use cached notes on subsequent renders", async () => {
      const mockNotes = [createMockInputNoteRecord("0xnote1")];

      const mockClient = createMockWebClient({
        getInputNotes: vi.fn().mockResolvedValue(mockNotes),
        getConsumableNotes: vi.fn().mockResolvedValue([]),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
      });

      act(() => {
        useMidenStore.getState().setClient(mockClient as any);
        useMidenStore.getState().setNotes(mockNotes as any);
      });

      const { result, rerender } = renderHook(() => useNotes());

      // Should already have notes from store
      expect(result.current.notes.length).toBe(1);

      // Rerender
      rerender();

      // Should not fetch again because notes already exist
      expect(mockClient.getInputNotes).not.toHaveBeenCalled();
    });
  });

  describe("filtering", () => {
    it("should apply status filter", async () => {
      const mockClient = createMockWebClient({
        getInputNotes: vi.fn().mockResolvedValue([]),
        getConsumableNotes: vi.fn().mockResolvedValue([]),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
      });

      act(() => {
        useMidenStore.getState().setClient(mockClient as any);
      });

      // Test each filter status
      const statuses = [
        "all",
        "consumed",
        "committed",
        "expected",
        "processing",
      ] as const;

      for (const status of statuses) {
        const { result } = renderHook(() => useNotes({ status }));

        await act(async () => {
          await result.current.refetch();
        });

        // Verify NoteFilter was created (via the mock)
        expect(mockClient.getInputNotes).toHaveBeenCalled();
      }
    });

    it("should filter consumable notes by account ID", async () => {
      const accountId = "0xaccount123";
      const mockClient = createMockWebClient({
        getInputNotes: vi.fn().mockResolvedValue([]),
        getConsumableNotes: vi.fn().mockResolvedValue([]),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
      });

      act(() => {
        useMidenStore.getState().setClient(mockClient as any);
      });

      const { result } = renderHook(() => useNotes({ accountId }));

      await act(async () => {
        await result.current.refetch();
      });

      // Should call getConsumableNotes with account ID
      expect(mockClient.getConsumableNotes).toHaveBeenCalled();
    });

    it("should fetch all consumable notes when no account filter", async () => {
      const mockClient = createMockWebClient({
        getInputNotes: vi.fn().mockResolvedValue([]),
        getConsumableNotes: vi.fn().mockResolvedValue([]),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
      });

      act(() => {
        useMidenStore.getState().setClient(mockClient as any);
      });

      const { result } = renderHook(() => useNotes());

      await act(async () => {
        await result.current.refetch();
      });

      expect(mockClient.getConsumableNotes).toHaveBeenCalled();
    });
  });

  describe("refetch", () => {
    it("should refetch notes when called", async () => {
      const initialNotes = [createMockInputNoteRecord("0xnote1")];
      const updatedNotes = [
        createMockInputNoteRecord("0xnote1"),
        createMockInputNoteRecord("0xnote2"),
      ];

      const mockClient = createMockWebClient({
        getInputNotes: vi
          .fn()
          .mockResolvedValueOnce(initialNotes)
          .mockResolvedValueOnce(updatedNotes),
        getConsumableNotes: vi.fn().mockResolvedValue([]),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
      });

      act(() => {
        useMidenStore.getState().setClient(mockClient as any);
      });

      const { result } = renderHook(() => useNotes());

      // Wait for auto-fetch on mount (since isReady is true and notes are empty)
      await waitFor(() => {
        expect(result.current.notes.length).toBe(1);
      });

      // Refetch to get updated notes
      await act(async () => {
        await result.current.refetch();
      });

      await waitFor(() => {
        expect(result.current.notes.length).toBe(2);
      });

      // Should have been called twice (auto-fetch + manual refetch)
      expect(mockClient.getInputNotes).toHaveBeenCalledTimes(2);
    });

    it("should not refetch when client is not ready", async () => {
      const mockClient = createMockWebClient();

      mockUseMiden.mockReturnValue({
        client: null,
        isReady: false,
      });

      const { result } = renderHook(() => useNotes());

      await act(async () => {
        await result.current.refetch();
      });

      expect(mockClient.getInputNotes).not.toHaveBeenCalled();
    });
  });

  describe("loading state", () => {
    it("should track loading state during fetch", async () => {
      let resolveInputNotes: (value: any[]) => void;
      let resolveConsumable: (value: any[]) => void;

      const inputNotesPromise = new Promise<any[]>((resolve) => {
        resolveInputNotes = resolve;
      });
      const consumablePromise = new Promise<any[]>((resolve) => {
        resolveConsumable = resolve;
      });

      const mockClient = createMockWebClient({
        getInputNotes: vi.fn().mockReturnValue(inputNotesPromise),
        getConsumableNotes: vi.fn().mockReturnValue(consumablePromise),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
      });

      act(() => {
        useMidenStore.getState().setClient(mockClient as any);
      });

      const { result } = renderHook(() => useNotes());

      // Trigger fetch
      act(() => {
        result.current.refetch();
      });

      await waitFor(() => {
        expect(result.current.isLoading).toBe(true);
      });

      // Resolve promises
      await act(async () => {
        resolveInputNotes!([]);
        resolveConsumable!([]);
      });

      await waitFor(() => {
        expect(result.current.isLoading).toBe(false);
      });
    });
  });

  describe("error handling", () => {
    it("should capture errors during fetch", async () => {
      const fetchError = new Error("Failed to fetch notes");

      const mockClient = createMockWebClient({
        getInputNotes: vi.fn().mockRejectedValue(fetchError),
        getConsumableNotes: vi.fn().mockResolvedValue([]),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
      });

      act(() => {
        useMidenStore.getState().setClient(mockClient as any);
      });

      const { result } = renderHook(() => useNotes());

      await act(async () => {
        await result.current.refetch();
      });

      expect(result.current.error).not.toBeNull();
      expect(result.current.error?.message).toBe("Failed to fetch notes");
      expect(result.current.isLoading).toBe(false);
    });

    it("should handle consumable notes fetch error", async () => {
      const mockClient = createMockWebClient({
        getInputNotes: vi.fn().mockResolvedValue([]),
        getConsumableNotes: vi
          .fn()
          .mockRejectedValue(new Error("Consumable error")),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
      });

      act(() => {
        useMidenStore.getState().setClient(mockClient as any);
      });

      const { result } = renderHook(() => useNotes());

      await act(async () => {
        await result.current.refetch();
      });

      expect(result.current.error).not.toBeNull();
    });
  });

  describe("note states", () => {
    it("should return notes with different states", async () => {
      const committedNote = createMockInputNoteRecord("0xnote1", false);
      const consumedNote = createMockInputNoteRecord("0xnote2", true);

      const mockClient = createMockWebClient({
        getInputNotes: vi.fn().mockResolvedValue([committedNote, consumedNote]),
        getConsumableNotes: vi.fn().mockResolvedValue([]),
      });

      mockUseMiden.mockReturnValue({
        client: mockClient,
        isReady: true,
      });

      act(() => {
        useMidenStore.getState().setClient(mockClient as any);
      });

      const { result } = renderHook(() => useNotes());

      await act(async () => {
        await result.current.refetch();
      });

      expect(result.current.notes.length).toBe(2);
    });
  });
});
