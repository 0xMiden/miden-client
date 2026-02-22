export function bytesToBigInt(bytes: Uint8Array): bigint {
  let result = 0n;
  for (let i = 0; i < bytes.length; i++) {
    result = (result << 8n) | BigInt(bytes[i]);
  }
  return result;
}

export function bigIntToBytes(value: bigint, length: number): Uint8Array {
  if (value < 0n) {
    throw new RangeError("bigIntToBytes: value must be non-negative");
  }
  const maxValue = (1n << BigInt(length * 8)) - 1n;
  if (value > maxValue) {
    throw new RangeError(
      `bigIntToBytes: value ${value} does not fit in ${length} byte(s) (max ${maxValue})`
    );
  }
  const bytes = new Uint8Array(length);
  let remaining = value;
  for (let i = length - 1; i >= 0; i--) {
    bytes[i] = Number(remaining & 0xffn);
    remaining >>= 8n;
  }
  return bytes;
}

export function concatBytes(...arrays: Uint8Array[]): Uint8Array {
  const totalLength = arrays.reduce((sum, arr) => sum + arr.length, 0);
  const result = new Uint8Array(totalLength);
  let offset = 0;
  for (const arr of arrays) {
    result.set(arr, offset);
    offset += arr.length;
  }
  return result;
}
