import { describe, it, expect } from 'vitest';
import { u8ToB64, b64ToU8 } from '../helpers';

describe('u8ToB64', () => {
  it('encodes a known value correctly', () => {
    const bytes = new Uint8Array([72, 101, 108, 108, 111]); // "Hello"
    expect(u8ToB64(bytes)).toBe('SGVsbG8=');
  });

  it('encodes an empty array', () => {
    expect(u8ToB64(new Uint8Array([]))).toBe('');
  });
});

describe('b64ToU8', () => {
  it('decodes a known value correctly', () => {
    const result = b64ToU8('SGVsbG8=');
    expect(result).toEqual(new Uint8Array([72, 101, 108, 108, 111]));
  });

  it('decodes an empty string', () => {
    expect(b64ToU8('')).toEqual(new Uint8Array([]));
  });
});

describe('roundtrip', () => {
  it('encode then decode returns original bytes', () => {
    const original = new Uint8Array([0, 1, 127, 128, 255]);
    const decoded = b64ToU8(u8ToB64(original));
    expect(decoded).toEqual(original);
  });

  it('handles a large array (256 bytes)', () => {
    const original = new Uint8Array(256);
    for (let i = 0; i < 256; i++) original[i] = i;
    const decoded = b64ToU8(u8ToB64(original));
    expect(decoded).toEqual(original);
  });
});
