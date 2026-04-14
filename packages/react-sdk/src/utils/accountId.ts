import { parseAccountId } from "./accountParsing";
import { toBech32AccountId } from "./accountBech32";

/**
 * Normalize any account ID format (hex, bech32, 0x-prefixed) to bech32.
 * Returns the original string if conversion fails.
 */
export function normalizeAccountId(id: string): string {
  return toBech32AccountId(id);
}

/**
 * Compare two account IDs for equality regardless of format (hex vs bech32).
 * Parses both to AccountId objects and compares their hex representations.
 */
export function accountIdsEqual(a: string, b: string): boolean {
  let idA: ReturnType<typeof parseAccountId> | undefined;
  let idB: ReturnType<typeof parseAccountId> | undefined;
  try {
    idA = parseAccountId(a);
    idB = parseAccountId(b);
    return idA.toString() === idB.toString();
  } catch {
    return a === b;
  } finally {
    // Free WASM objects to prevent memory leaks
    (idA as { free?: () => void })?.free?.();
    (idB as { free?: () => void })?.free?.();
  }
}
