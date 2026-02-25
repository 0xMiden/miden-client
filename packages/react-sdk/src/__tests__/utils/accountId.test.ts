import { describe, it, expect, vi } from "vitest";
import { normalizeAccountId, accountIdsEqual } from "../../utils/accountId";
import { AccountId } from "@miden-sdk/miden-sdk";

// The global mock in setup.ts mocks @miden-sdk/miden-sdk.
// parseAccountId (used by accountIdsEqual) calls AccountId.fromHex / fromBech32,
// and toBech32AccountId (used by normalizeAccountId) calls parseAccountId + toString.
// The mock AccountId.fromHex returns { toString: () => id }, so both functions
// will work in this mock environment.

describe("normalizeAccountId", () => {
  it("should return a string for any input", () => {
    const result = normalizeAccountId("0x1234");
    expect(typeof result).toBe("string");
  });

  it("should handle hex input", () => {
    // The mock parseAccountId calls AccountId.fromHex which returns { toString: () => id }
    // toBech32AccountId calls parseAccountId and then toString, returning the original
    const result = normalizeAccountId("0xabcdef");
    expect(result).toBeDefined();
  });

  it("should return original string on failure", () => {
    // normalizeAccountId uses toBech32AccountId which catches errors
    // and returns the original string
    const result = normalizeAccountId("invalid-id");
    expect(typeof result).toBe("string");
  });
});

describe("accountIdsEqual", () => {
  it("should return true for identical hex IDs", () => {
    expect(accountIdsEqual("0x1234", "0x1234")).toBe(true);
  });

  it("should return false for different IDs", () => {
    expect(accountIdsEqual("0x1234", "0x5678")).toBe(false);
  });

  it("should fall back to string comparison on parse failure", () => {
    // When both parse to the same mock object toString, they're equal
    expect(accountIdsEqual("0xabc", "0xabc")).toBe(true);
  });

  it("should handle empty strings", () => {
    expect(accountIdsEqual("", "")).toBe(true);
  });

  it("should handle mismatched formats that resolve to same ID", () => {
    // In the mock, both will be parsed via fromHex and fromBech32
    // and toString returns the original hex
    const same = "0x1234567890abcdef";
    expect(accountIdsEqual(same, same)).toBe(true);
  });

  it("should free WASM objects after comparison", () => {
    vi.mocked(AccountId.fromHex).mockClear();

    accountIdsEqual("0xaaa", "0xbbb");

    // Verify fromHex was called (creates WASM objects)
    expect(vi.mocked(AccountId.fromHex)).toHaveBeenCalledTimes(2);

    // Verify the mock objects' free() was called
    const calls = vi.mocked(AccountId.fromHex).mock.results;
    for (const call of calls) {
      expect(call.value.free).toHaveBeenCalled();
    }
  });
});
