import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import { useTransactionHistory } from "../../hooks/useTransactionHistory";
import { useMiden } from "../../context/MidenProvider";
import { useMidenStore } from "../../store/MidenStore";
import { TransactionFilter } from "@miden-sdk/miden-sdk";
import {
  createMockTransactionId,
  createMockWebClient,
} from "../mocks/miden-sdk";

vi.mock("../../context/MidenProvider", () => ({
  useMiden: vi.fn(),
}));

const mockUseMiden = useMiden as ReturnType<typeof vi.fn>;

beforeEach(() => {
  useMidenStore.getState().reset();
  vi.clearAllMocks();
});

const createRecord = (
  id: string,
  status: "committed" | "pending" | "discarded" = "committed"
) => ({
  id: vi.fn(() => createMockTransactionId(id)),
  transactionStatus: vi.fn(() => ({
    isPending: vi.fn(() => status === "pending"),
    isCommitted: vi.fn(() => status === "committed"),
    isDiscarded: vi.fn(() => status === "discarded"),
  })),
});

describe("useTransactionHistory", () => {
  it("fetches all transactions by default", async () => {
    const record = createRecord("0xabc");
    const mockClient = createMockWebClient({
      getTransactions: vi.fn().mockResolvedValue([record]),
    });

    mockUseMiden.mockReturnValue({
      client: mockClient,
      isReady: true,
    });

    const { result } = renderHook(() => useTransactionHistory());

    await waitFor(() => expect(result.current.records).toHaveLength(1));

    expect(TransactionFilter.all).toHaveBeenCalled();
  });

  it("uses TransactionFilter.ids when a TransactionId is provided", async () => {
    const txId = createMockTransactionId("0xdeadbeef");
    const record = createRecord("0xdeadbeef", "committed");
    const mockClient = createMockWebClient({
      getTransactions: vi.fn().mockResolvedValue([record]),
    });

    mockUseMiden.mockReturnValue({
      client: mockClient,
      isReady: true,
    });

    const { result } = renderHook(() =>
      useTransactionHistory({ id: txId })
    );

    await waitFor(() => expect(result.current.record).not.toBeNull());

    expect(TransactionFilter.ids).toHaveBeenCalledWith([txId]);
    expect(result.current.status).toBe("committed");
  });

  it("filters by string ids locally", async () => {
    const recordA = createRecord("0xaaa", "pending");
    const recordB = createRecord("0xbbb", "discarded");
    const mockClient = createMockWebClient({
      getTransactions: vi.fn().mockResolvedValue([recordA, recordB]),
    });

    mockUseMiden.mockReturnValue({
      client: mockClient,
      isReady: true,
    });

    const { result } = renderHook(() =>
      useTransactionHistory({ ids: ["0xbbb"] })
    );

    await waitFor(() => expect(result.current.records).toHaveLength(1));

    expect(TransactionFilter.all).toHaveBeenCalled();
    expect(result.current.records[0]?.id().toHex()).toBe("0xbbb");
    expect(result.current.status).toBe("discarded");
  });
});
