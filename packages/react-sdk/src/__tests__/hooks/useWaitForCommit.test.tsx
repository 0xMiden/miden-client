import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook } from "@testing-library/react";
import { useWaitForCommit } from "../../hooks/useWaitForCommit";
import { useMiden } from "../../context/MidenProvider";
import { useMidenStore } from "../../store/MidenStore";
import { TransactionFilter } from "@miden-sdk/miden-sdk";
import {
  createMockWebClient,
  createMockTransactionId,
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

describe("useWaitForCommit", () => {
  it("should throw when client is not ready", async () => {
    mockUseMiden.mockReturnValue({
      client: null,
      isReady: false,
    });

    const { result } = renderHook(() => useWaitForCommit());

    await expect(result.current.waitForCommit("0xtx")).rejects.toThrow(
      "Miden client is not ready"
    );
  });

  it("should resolve when transaction is committed (string id)", async () => {
    const record = {
      id: vi.fn(() => ({ toHex: () => "0xtx" })),
      transactionStatus: vi.fn(() => ({
        isPending: vi.fn(() => false),
        isCommitted: vi.fn(() => true),
        isDiscarded: vi.fn(() => false),
      })),
    };

    const mockClient = createMockWebClient({
      syncState: vi.fn().mockResolvedValue({}),
      getTransactions: vi.fn().mockResolvedValue([record]),
    });

    mockUseMiden.mockReturnValue({
      client: mockClient,
      isReady: true,
    });

    const { result } = renderHook(() => useWaitForCommit());

    await result.current.waitForCommit("0xtx", {
      timeoutMs: 20,
      intervalMs: 1,
    });

    expect(mockClient.getTransactions).toHaveBeenCalledWith(
      TransactionFilter.all()
    );
  });

  it("should resolve when transaction is committed (TransactionId)", async () => {
    const txId = createMockTransactionId("0xtx123");
    const record = {
      id: vi.fn(() => ({ toHex: () => "0xtx123" })),
      transactionStatus: vi.fn(() => ({
        isPending: vi.fn(() => false),
        isCommitted: vi.fn(() => true),
        isDiscarded: vi.fn(() => false),
      })),
    };

    const mockClient = createMockWebClient({
      syncState: vi.fn().mockResolvedValue({}),
      getTransactions: vi.fn().mockResolvedValue([record]),
    });

    mockUseMiden.mockReturnValue({
      client: mockClient,
      isReady: true,
    });

    const { result } = renderHook(() => useWaitForCommit());

    await result.current.waitForCommit(txId, {
      timeoutMs: 20,
      intervalMs: 1,
    });

    expect(mockClient.getTransactions).toHaveBeenCalledWith(
      TransactionFilter.ids([txId])
    );
  });

  it("should throw when transaction is discarded", async () => {
    const record = {
      id: vi.fn(() => ({ toHex: () => "0xtx" })),
      transactionStatus: vi.fn(() => ({
        isPending: vi.fn(() => false),
        isCommitted: vi.fn(() => false),
        isDiscarded: vi.fn(() => true),
      })),
    };

    const mockClient = createMockWebClient({
      syncState: vi.fn().mockResolvedValue({}),
      getTransactions: vi.fn().mockResolvedValue([record]),
    });

    mockUseMiden.mockReturnValue({
      client: mockClient,
      isReady: true,
    });

    const { result } = renderHook(() => useWaitForCommit());

    await expect(
      result.current.waitForCommit("0xtx", { timeoutMs: 20, intervalMs: 1 })
    ).rejects.toThrow("Transaction was discarded before commit");
  });
});
