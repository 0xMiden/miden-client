import { describe, it, expect } from "vitest";
import { bytesToBigInt, bigIntToBytes, concatBytes } from "../../utils/bytes";

describe("bytesToBigInt", () => {
  it("should convert empty array to 0n", () => {
    expect(bytesToBigInt(new Uint8Array([]))).toBe(0n);
  });

  it("should convert single byte", () => {
    expect(bytesToBigInt(new Uint8Array([0xff]))).toBe(255n);
  });

  it("should convert two bytes (big-endian)", () => {
    expect(bytesToBigInt(new Uint8Array([0x01, 0x00]))).toBe(256n);
  });

  it("should convert 8 bytes", () => {
    const bytes = new Uint8Array([0, 0, 0, 0, 0, 0, 1, 0]);
    expect(bytesToBigInt(bytes)).toBe(256n);
  });

  it("should handle max u64", () => {
    const bytes = new Uint8Array([0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]);
    expect(bytesToBigInt(bytes)).toBe(2n ** 64n - 1n);
  });

  it("should handle zero bytes", () => {
    expect(bytesToBigInt(new Uint8Array([0, 0, 0]))).toBe(0n);
  });
});

describe("bigIntToBytes", () => {
  it("should convert 0n to zero-filled array", () => {
    expect(bigIntToBytes(0n, 4)).toEqual(new Uint8Array([0, 0, 0, 0]));
  });

  it("should convert 255n to single byte", () => {
    expect(bigIntToBytes(255n, 1)).toEqual(new Uint8Array([0xff]));
  });

  it("should convert 256n to two bytes (big-endian)", () => {
    expect(bigIntToBytes(256n, 2)).toEqual(new Uint8Array([0x01, 0x00]));
  });

  it("should pad with leading zeros", () => {
    expect(bigIntToBytes(1n, 4)).toEqual(new Uint8Array([0, 0, 0, 1]));
  });

  it("should throw RangeError when value overflows length", () => {
    // 256n = 0x0100, doesn't fit in 1 byte (max 255)
    expect(() => bigIntToBytes(256n, 1)).toThrow(RangeError);
    expect(() => bigIntToBytes(256n, 1)).toThrow("does not fit in 1 byte(s)");
  });

  it("should throw RangeError for negative values", () => {
    expect(() => bigIntToBytes(-1n, 4)).toThrow(RangeError);
    expect(() => bigIntToBytes(-1n, 4)).toThrow("non-negative");
  });

  it("should roundtrip with bytesToBigInt", () => {
    const value = 123456789n;
    const bytes = bigIntToBytes(value, 8);
    expect(bytesToBigInt(bytes)).toBe(value);
  });
});

describe("concatBytes", () => {
  it("should return empty array for no arguments", () => {
    expect(concatBytes()).toEqual(new Uint8Array([]));
  });

  it("should return copy of single array", () => {
    const a = new Uint8Array([1, 2, 3]);
    const result = concatBytes(a);
    expect(result).toEqual(new Uint8Array([1, 2, 3]));
    expect(result).not.toBe(a); // Must be a new array
  });

  it("should concatenate two arrays", () => {
    const a = new Uint8Array([1, 2]);
    const b = new Uint8Array([3, 4]);
    expect(concatBytes(a, b)).toEqual(new Uint8Array([1, 2, 3, 4]));
  });

  it("should concatenate three arrays", () => {
    const a = new Uint8Array([1]);
    const b = new Uint8Array([2, 3]);
    const c = new Uint8Array([4, 5, 6]);
    expect(concatBytes(a, b, c)).toEqual(new Uint8Array([1, 2, 3, 4, 5, 6]));
  });

  it("should handle empty arrays in the mix", () => {
    const a = new Uint8Array([]);
    const b = new Uint8Array([1, 2]);
    const c = new Uint8Array([]);
    expect(concatBytes(a, b, c)).toEqual(new Uint8Array([1, 2]));
  });
});
