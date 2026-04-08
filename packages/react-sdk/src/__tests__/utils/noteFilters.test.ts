import { describe, it, expect, vi } from "vitest";
import {
  getNoteFilterType,
  waitForTransactionCommit,
} from "../../utils/noteFilters";
import { NoteFilterTypes } from "@miden-sdk/miden-sdk";

describe("getNoteFilterType", () => {
  it("should return All for undefined status", () => {
    expect(getNoteFilterType(undefined)).toBe(NoteFilterTypes.All);
  });

  it("should return All for 'all' status", () => {
    expect(getNoteFilterType("all")).toBe(NoteFilterTypes.All);
  });

  it("should return Consumed for 'consumed' status", () => {
    expect(getNoteFilterType("consumed")).toBe(NoteFilterTypes.Consumed);
  });

  it("should return Committed for 'committed' status", () => {
    expect(getNoteFilterType("committed")).toBe(NoteFilterTypes.Committed);
  });

  it("should return Expected for 'expected' status", () => {
    expect(getNoteFilterType("expected")).toBe(NoteFilterTypes.Expected);
  });

  it("should return Processing for 'processing' status", () => {
    expect(getNoteFilterType("processing")).toBe(NoteFilterTypes.Processing);
  });
});

describe("waitForTransactionCommit", () => {
  // Uses real timers with very short delays to avoid fake-timer / Date.now()
  // interaction issues that cause unhandled rejection warnings.

  const TX_HEX = "0xtx123";

  const createMockClient = (
    statusSequence: Array<"pending" | "committed" | "discarded"> = ["committed"]
  ) => {
    let callIndex = 0;
    return {
      syncState: vi.fn().mockResolvedValue(undefined),
      getTransactions: vi.fn().mockImplementation(() => {
        const status = statusSequence[callIndex] ?? "pending";
        callIndex++;
        return Promise.resolve([
          {
            id: vi.fn(() => ({ toHex: vi.fn(() => TX_HEX) })),
            transactionStatus: vi.fn(() => ({
              isPending: vi.fn(() => status === "pending"),
              isCommitted: vi.fn(() => status === "committed"),
              isDiscarded: vi.fn(() => status === "discarded"),
            })),
          },
        ]);
      }),
    };
  };

  const mockRunExclusive = async <T>(fn: () => Promise<T>): Promise<T> => fn();

  it("should resolve when transaction is committed on first check", async () => {
    const client = createMockClient(["committed"]);

    await waitForTransactionCommit(
      client as never,
      mockRunExclusive,
      TX_HEX,
      5000,
      10
    );

    expect(client.syncState).toHaveBeenCalledTimes(1);
    expect(client.getTransactions).toHaveBeenCalledTimes(1);
  });

  it("should poll until transaction is committed", async () => {
    const client = createMockClient(["pending", "pending", "committed"]);

    await waitForTransactionCommit(
      client as never,
      mockRunExclusive,
      TX_HEX,
      5000,
      10
    );

    expect(client.syncState).toHaveBeenCalledTimes(3);
    expect(client.getTransactions).toHaveBeenCalledTimes(3);
  });

  it("should throw on discarded transaction", async () => {
    const client = createMockClient(["discarded"]);

    await expect(
      waitForTransactionCommit(
        client as never,
        mockRunExclusive,
        TX_HEX,
        5000,
        10
      )
    ).rejects.toThrow("Transaction was discarded before commit");
  });

  it("should throw on timeout", async () => {
    const client = createMockClient([
      "pending",
      "pending",
      "pending",
      "pending",
      "pending",
      "pending",
      "pending",
      "pending",
      "pending",
      "pending",
    ]);

    await expect(
      waitForTransactionCommit(
        client as never,
        mockRunExclusive,
        TX_HEX,
        100,
        10
      )
    ).rejects.toThrow("Timeout waiting for transaction commit");
  });

  it("should match transaction by hex regardless of case/prefix", async () => {
    const client = createMockClient(["committed"]);

    // Pass uppercase hex without 0x prefix — should still match "0xtx123"
    await waitForTransactionCommit(
      client as never,
      mockRunExclusive,
      "TX123",
      5000,
      10
    );

    expect(client.getTransactions).toHaveBeenCalledTimes(1);
  });

  it("should use wall-clock time including async operation duration", async () => {
    const client = {
      syncState: vi
        .fn()
        .mockImplementation(
          () => new Promise((resolve) => setTimeout(resolve, 30))
        ),
      getTransactions: vi.fn().mockResolvedValue([
        {
          id: vi.fn(() => ({ toHex: vi.fn(() => TX_HEX) })),
          transactionStatus: vi.fn(() => ({
            isPending: vi.fn(() => true),
            isCommitted: vi.fn(() => false),
            isDiscarded: vi.fn(() => false),
          })),
        },
      ]),
    };

    await expect(
      waitForTransactionCommit(
        client as never,
        mockRunExclusive,
        TX_HEX,
        150,
        10
      )
    ).rejects.toThrow("Timeout waiting for transaction commit");

    // Each iteration takes ~40ms (30ms sync + 10ms delay), so with wall-clock
    // time we should complete fewer iterations than 150/10 = 15
    expect(client.syncState.mock.calls.length).toBeLessThanOrEqual(5);
  });
});
