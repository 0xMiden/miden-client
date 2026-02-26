/**
 * Encryption layer for secret keys stored in IndexedDB.
 *
 * Threat model: the encryption key is stored in the same IDB database as the
 * encrypted data. This means a determined same-origin attacker who can read
 * arbitrary IDB tables can also obtain the key. The protection this provides:
 *
 * 1. The key is created with `extractable: false`, so JS code cannot call
 *    `crypto.subtle.exportKey()` to read the raw key material — only the
 *    browser's native crypto implementation can use it.
 * 2. Secrets are not stored as plaintext strings, preventing casual inspection
 *    via browser DevTools, IDB viewers, or naive scraping of string values.
 *
 * For stronger protection, the key should be derived from user-provided
 * credentials (e.g. a password via PBKDF2) in a future iteration.
 *
 * Note: the CryptoKey is non-exportable and does NOT survive a full store
 * export/import cycle. The export function decrypts auth records to plaintext
 * before serializing so that imports can re-encrypt with a fresh key.
 */
import { getDatabase } from "./schema.js";

const keyCache = new Map<string, CryptoKey>();

async function getOrCreateEncryptionKey(dbId: string): Promise<CryptoKey> {
  const cached = keyCache.get(dbId);
  if (cached) return cached;

  const db = getDatabase(dbId);
  // CryptoKey objects are structured-cloneable and can be stored directly in IDB.
  // ISetting.value is typed as Uint8Array, so we use the raw Dexie table API
  // to store the opaque CryptoKey directly.
  const stored = (await db.dexie
    .table("settings")
    .get("encryptionKey")) as { key: string; value: unknown } | undefined;
  let key: CryptoKey;

  if (stored && stored.value instanceof CryptoKey) {
    key = stored.value;
  } else {
    // extractable: false — key cannot be exported, only used for encrypt/decrypt
    key = await crypto.subtle.generateKey(
      { name: "AES-GCM", length: 256 },
      false,
      ["encrypt", "decrypt"]
    );
    // Store the opaque CryptoKey directly in IDB (structured clone)
    await db.dexie.table("settings").put({ key: "encryptionKey", value: key });
  }

  keyCache.set(dbId, key);
  return key;
}

export async function encryptSecretKey(
  dbId: string,
  plaintext: string
): Promise<{ encrypted: Uint8Array; iv: Uint8Array }> {
  const key = await getOrCreateEncryptionKey(dbId);
  const iv = crypto.getRandomValues(new Uint8Array(12));
  const encoded = new TextEncoder().encode(plaintext);
  const encrypted = new Uint8Array(
    await crypto.subtle.encrypt({ name: "AES-GCM", iv }, key, encoded)
  );
  return { encrypted, iv };
}

export async function decryptSecretKey(
  dbId: string,
  encrypted: Uint8Array,
  iv: Uint8Array
): Promise<string> {
  const key = await getOrCreateEncryptionKey(dbId);
  // Wrap in new Uint8Array() to get Uint8Array<ArrayBuffer> (required by
  // TypeScript 5.7+ for BufferSource compatibility).
  const decrypted = await crypto.subtle.decrypt(
    { name: "AES-GCM", iv: new Uint8Array(iv) },
    key,
    new Uint8Array(encrypted)
  );
  return new TextDecoder().decode(decrypted);
}
