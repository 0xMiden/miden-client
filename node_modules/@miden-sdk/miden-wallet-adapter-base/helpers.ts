// Encode a Uint8Array to base64
export function u8ToB64(u8: Uint8Array): string {
  // Prefer browser builtins (no polyfills)
  if (typeof globalThis.btoa === 'function') {
    let s = '';
    for (let i = 0; i < u8.length; i++) s += String.fromCharCode(u8[i]);
    return globalThis.btoa(s);
  }
  // Node/Buffer fallback (won’t be bundled if never executed in browsers)
  const Buf: any = (globalThis as any).Buffer;
  if (Buf) return Buf.from(u8).toString('base64');
  throw new Error('No base64 encoder available in this environment');
}

// Decode base64 to Uint8Array
export function b64ToU8(b64: string): Uint8Array {
  // Prefer browser builtins (no polyfills)
  if (typeof globalThis.atob === 'function') {
    const bin = globalThis.atob(b64);
    const out = new Uint8Array(bin.length);
    for (let i = 0; i < bin.length; i++) out[i] = bin.charCodeAt(i);
    return out;
  }
  // Node/Buffer fallback (won’t be bundled if never executed in browsers)
  const Buf: any = (globalThis as any).Buffer;
  if (Buf) return new Uint8Array(Buf.from(b64, 'base64'));
  throw new Error('No base64 decoder available in this environment');
}