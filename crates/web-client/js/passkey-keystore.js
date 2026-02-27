/**
 * WebAuthn PRF-based key encryption for Miden web client.
 *
 * Provides opt-in passkey encryption for secret keys at rest using the WebAuthn
 * PRF extension (Touch ID, Face ID, Windows Hello). Keys are wrapped with
 * AES-256-GCM using a wrapping key derived from the authenticator's PRF output.
 *
 * Browser support: Chrome 116+, Safari 18+, Edge 116+. Firefox does NOT support PRF.
 *
 * @module passkey-keystore
 */

import Dexie from "dexie";

// ════════════════════════════════════════════════════════════════
// Constants
// ════════════════════════════════════════════════════════════════

/** Magic bytes for web-encrypted payloads. Distinct from native "MENC" (ChaCha20-Poly1305). */
const ENCRYPTED_MAGIC = new Uint8Array([0x4d, 0x57, 0x45, 0x42]); // "MWEB"

/** Format version byte. Bump on breaking format changes. */
const ENCRYPTED_VERSION = 0x01;

/** Fixed HKDF salt — must never change (needed to reproduce wrapping key across sessions). */
const HKDF_SALT = new TextEncoder().encode("miden-client-passkey-v1");

/** Fixed HKDF info — domain separation for the wrapping key derivation. */
const HKDF_INFO = new TextEncoder().encode("aes-256-gcm-wrapping-key");

/** Fixed salt for WebAuthn PRF evaluation. */
const PRF_EVAL_SALT = new TextEncoder().encode("miden-passkey-prf-salt-v1");

/** AES-GCM IV length in bytes. */
const IV_LEN = 12;

/** Header length: magic (4) + version (1) + IV (12) = 17 bytes. */
const HEADER_LEN = 4 + 1 + IV_LEN;

/** localStorage key prefix for credential IDs. */
const CREDENTIAL_STORAGE_PREFIX = "miden_passkey_credential_";

// ════════════════════════════════════════════════════════════════
// Feature detection
// ════════════════════════════════════════════════════════════════

/**
 * Returns `true` if the current browser supports WebAuthn with the PRF extension.
 *
 * Checks for:
 * 1. `PublicKeyCredential` API availability
 * 2. Platform authenticator availability
 * 3. PRF extension support (via `getClientCapabilities` or fallback heuristic)
 *
 * @returns {Promise<boolean>}
 */
export async function isPasskeyPrfSupported() {
  try {
    if (
      typeof window === "undefined" ||
      !window.PublicKeyCredential ||
      !navigator.credentials
    ) {
      return false;
    }

    // Check platform authenticator
    if (
      typeof PublicKeyCredential.isUserVerifyingPlatformAuthenticatorAvailable ===
      "function"
    ) {
      const available =
        await PublicKeyCredential.isUserVerifyingPlatformAuthenticatorAvailable();
      if (!available) return false;
    }

    // Prefer getClientCapabilities if available (Chrome 132+)
    if (typeof PublicKeyCredential.getClientCapabilities === "function") {
      const capabilities = await PublicKeyCredential.getClientCapabilities();
      return capabilities?.["extension:prf"] === true;
    }

    // Fallback: check user agent for known-supporting browsers
    const ua = navigator.userAgent;
    // Chrome/Edge 116+ support PRF
    const chromeMatch = ua.match(/Chrom(?:e|ium)\/(\d+)/);
    if (chromeMatch && parseInt(chromeMatch[1], 10) >= 116) return true;
    // Safari 18+ supports PRF
    const safariMatch = ua.match(/Version\/(\d+).*Safari/);
    if (safariMatch && parseInt(safariMatch[1], 10) >= 18) return true;

    return false;
  } catch {
    return false;
  }
}

// ════════════════════════════════════════════════════════════════
// WebAuthn operations
// ════════════════════════════════════════════════════════════════

/**
 * Registers a new passkey with PRF extension support.
 *
 * @param {object} options
 * @param {string} [options.rpId] - Relying party ID. Defaults to current hostname.
 * @param {string} [options.rpName] - Relying party display name. Defaults to "Miden Client".
 * @param {string} [options.userName] - User display name. Defaults to "Miden Wallet User".
 * @returns {Promise<{ credentialId: string, prfOutput: ArrayBuffer }>}
 */
async function registerPasskey(options = {}) {
  const rpId = options.rpId || window.location.hostname;
  const rpName = options.rpName || "Miden Client";
  const userName = options.userName || "Miden Wallet User";

  // Generate a random user ID (not secret, just needs to be unique)
  const userId = crypto.getRandomValues(new Uint8Array(32));

  const credential = await navigator.credentials.create({
    publicKey: {
      rp: { id: rpId, name: rpName },
      user: {
        id: userId,
        name: userName,
        displayName: userName,
      },
      challenge: crypto.getRandomValues(new Uint8Array(32)),
      pubKeyCredParams: [
        { alg: -7, type: "public-key" }, // ES256
        { alg: -257, type: "public-key" }, // RS256
      ],
      authenticatorSelection: {
        authenticatorAttachment: "platform",
        residentKey: "required",
        userVerification: "required",
      },
      extensions: {
        prf: {
          eval: { first: PRF_EVAL_SALT },
        },
      },
    },
  });

  const prfResults = credential.getClientExtensionResults()?.prf;
  if (!prfResults?.results?.first) {
    throw new Error(
      "WebAuthn PRF extension not supported by this authenticator. " +
        "Use isPasskeyPrfSupported() to check before enabling passkey encryption."
    );
  }

  const credentialId = bufferToBase64Url(credential.rawId);
  return {
    credentialId,
    prfOutput: prfResults.results.first,
  };
}

/**
 * Authenticates with an existing passkey and evaluates the PRF extension.
 *
 * @param {string} credentialId - Base64url-encoded credential ID.
 * @param {string} [rpId] - Relying party ID. Defaults to current hostname.
 * @returns {Promise<ArrayBuffer>} The PRF output (32 bytes).
 */
async function authenticateWithPrf(credentialId, rpId) {
  rpId = rpId || window.location.hostname;
  const credentialIdBuffer = base64UrlToBuffer(credentialId);

  const assertion = await navigator.credentials.get({
    publicKey: {
      challenge: crypto.getRandomValues(new Uint8Array(32)),
      rpId,
      allowCredentials: [
        {
          id: credentialIdBuffer,
          type: "public-key",
          transports: ["internal"],
        },
      ],
      userVerification: "required",
      extensions: {
        prf: {
          eval: { first: PRF_EVAL_SALT },
        },
      },
    },
  });

  const prfResults = assertion.getClientExtensionResults()?.prf;
  if (!prfResults?.results?.first) {
    throw new Error(
      "PRF evaluation failed. The authenticator did not return a PRF result."
    );
  }

  return prfResults.results.first;
}

// ════════════════════════════════════════════════════════════════
// Key derivation & encryption
// ════════════════════════════════════════════════════════════════

/**
 * Derives a non-extractable AES-256-GCM wrapping key from PRF output via HKDF-SHA256.
 *
 * @param {ArrayBuffer} prfOutput - Raw PRF output from the authenticator.
 * @returns {Promise<CryptoKey>} Non-extractable AES-256-GCM key.
 */
async function deriveWrappingKey(prfOutput) {
  if (prfOutput.byteLength < 32) {
    throw new Error(
      `PRF output too short: expected at least 32 bytes, got ${prfOutput.byteLength}. ` +
        "The authenticator may not support sufficient entropy for key derivation."
    );
  }

  const keyMaterial = await crypto.subtle.importKey(
    "raw",
    prfOutput,
    "HKDF",
    false,
    ["deriveKey"]
  );

  return crypto.subtle.deriveKey(
    {
      name: "HKDF",
      hash: "SHA-256",
      salt: HKDF_SALT,
      info: HKDF_INFO,
    },
    keyMaterial,
    { name: "AES-GCM", length: 256 },
    false, // non-extractable
    ["encrypt", "decrypt"]
  );
}

/**
 * Encrypts a secret key using AES-256-GCM with the MWEB envelope format.
 *
 * Format: [4B: "MWEB"] [1B: version=0x01] [12B: IV] [NB: ciphertext + 16B auth tag]
 *
 * The pub key commitment bytes are used as AES-GCM additional authenticated data (AAD),
 * binding each ciphertext to its corresponding public key. This prevents ciphertext-swapping
 * attacks where an attacker rearranges entries in IndexedDB.
 *
 * @param {CryptoKey} wrappingKey - AES-256-GCM wrapping key.
 * @param {Uint8Array} plaintext - Secret key bytes to encrypt.
 * @param {Uint8Array} pubKeyCommitment - Public key commitment bytes (used as AAD).
 * @returns {Promise<Uint8Array>} MWEB-envelope ciphertext.
 */
async function encryptSecretKey(wrappingKey, plaintext, pubKeyCommitment) {
  const iv = crypto.getRandomValues(new Uint8Array(IV_LEN));

  const ciphertext = await crypto.subtle.encrypt(
    {
      name: "AES-GCM",
      iv,
      additionalData: pubKeyCommitment,
    },
    wrappingKey,
    plaintext
  );

  // Assemble: magic + version + iv + ciphertext
  const output = new Uint8Array(HEADER_LEN + ciphertext.byteLength);
  output.set(ENCRYPTED_MAGIC, 0);
  output[4] = ENCRYPTED_VERSION;
  output.set(iv, 5);
  output.set(new Uint8Array(ciphertext), HEADER_LEN);

  return output;
}

/**
 * Decrypts an MWEB-envelope ciphertext using AES-256-GCM.
 *
 * @param {CryptoKey} wrappingKey - AES-256-GCM wrapping key.
 * @param {Uint8Array} envelope - MWEB-envelope ciphertext.
 * @param {Uint8Array} pubKeyCommitment - Public key commitment bytes (used as AAD).
 * @returns {Promise<Uint8Array>} Decrypted secret key bytes.
 */
async function decryptSecretKey(wrappingKey, envelope, pubKeyCommitment) {
  if (envelope.length < HEADER_LEN + 16) {
    throw new Error("Encrypted payload too short");
  }

  // Validate magic
  if (
    envelope[0] !== ENCRYPTED_MAGIC[0] ||
    envelope[1] !== ENCRYPTED_MAGIC[1] ||
    envelope[2] !== ENCRYPTED_MAGIC[2] ||
    envelope[3] !== ENCRYPTED_MAGIC[3]
  ) {
    throw new Error("Invalid encrypted payload: bad magic bytes");
  }

  // Validate version
  const version = envelope[4];
  if (version !== ENCRYPTED_VERSION) {
    throw new Error(
      `Unsupported encryption format version: ${version}. ` +
        `This client supports version ${ENCRYPTED_VERSION}. ` +
        "You may need to update the SDK."
    );
  }

  const iv = envelope.slice(5, 5 + IV_LEN);
  const ciphertext = envelope.slice(HEADER_LEN);

  const plaintext = await crypto.subtle.decrypt(
    {
      name: "AES-GCM",
      iv,
      additionalData: pubKeyCommitment,
    },
    wrappingKey,
    ciphertext
  );

  return new Uint8Array(plaintext);
}

// ════════════════════════════════════════════════════════════════
// Credential persistence
// ════════════════════════════════════════════════════════════════

function getStoredCredentialId(storeName) {
  return localStorage.getItem(CREDENTIAL_STORAGE_PREFIX + storeName);
}

function storeCredentialId(storeName, credentialId) {
  localStorage.setItem(CREDENTIAL_STORAGE_PREFIX + storeName, credentialId);
}

// ════════════════════════════════════════════════════════════════
// Encrypted key storage (separate Dexie database)
// ════════════════════════════════════════════════════════════════

/**
 * Opens (or creates) the encrypted keystore database.
 *
 * Uses a separate Dexie database `MidenKeystore_${storeName}` to avoid
 * schema conflicts with the main `MidenClientDB` database.
 *
 * @param {string} storeName
 * @returns {Dexie}
 */
function openKeystoreDb(storeName) {
  const db = new Dexie(`MidenKeystore_${storeName}`);
  db.version(1).stores({
    keys: "pubKeyHex",
  });
  return db;
}

/**
 * Opens the main client Dexie database for migration fallback reads.
 *
 * This reads directly from the idxdb-store's `accountAuths` table, which
 * stores plaintext secret keys keyed by `pubKeyCommitmentHex`.
 *
 * The DB name must match what the WASM/Rust side uses. When storeName is
 * explicitly provided by the user, it is used as-is. We cannot determine
 * the auto-generated name (`MidenClientDB_{network_id}`) from JS because
 * that requires the WASM client to have been initialized first.
 *
 * @param {string} storeName - The store name as passed to MidenClient.create().
 * @returns {Dexie | null} The Dexie DB, or null if storeName is unknown.
 */
function openMainDbForMigration(storeName) {
  if (!storeName) return null;
  const db = new Dexie(storeName);
  // Declare just the table we need — Dexie allows partial schema declarations
  // for read-only access to existing databases.
  db.version(1).stores({
    accountAuths: "pubKeyCommitmentHex",
  });
  return db;
}

// ════════════════════════════════════════════════════════════════
// Base64url utilities
// ════════════════════════════════════════════════════════════════

function bufferToBase64Url(buffer) {
  const bytes = new Uint8Array(buffer);
  let binary = "";
  for (let i = 0; i < bytes.length; i++) {
    binary += String.fromCharCode(bytes[i]);
  }
  return btoa(binary).replace(/\+/g, "-").replace(/\//g, "_").replace(/=/g, "");
}

function base64UrlToBuffer(base64url) {
  const base64 = base64url.replace(/-/g, "+").replace(/_/g, "/");
  const padded = base64 + "=".repeat((4 - (base64.length % 4)) % 4);
  const binary = atob(padded);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) {
    bytes[i] = binary.charCodeAt(i);
  }
  return bytes.buffer;
}

function bytesToHex(bytes) {
  return Array.from(bytes)
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("");
}

function hexToBytes(hex) {
  if (hex.length % 2 !== 0) {
    throw new Error("Hex string must have even length");
  }
  const bytes = new Uint8Array(hex.length / 2);
  for (let i = 0; i < bytes.length; i++) {
    const byte = parseInt(hex.slice(i * 2, i * 2 + 2), 16);
    if (Number.isNaN(byte)) {
      throw new Error(`Invalid hex character at position ${i * 2}`);
    }
    bytes[i] = byte;
  }
  return bytes;
}

// ════════════════════════════════════════════════════════════════
// Factory
// ════════════════════════════════════════════════════════════════

/**
 * Creates a passkey-encrypted keystore backed by WebAuthn PRF.
 *
 * If no `credentialId` is provided and none exists in localStorage for the given
 * store name, a new passkey is registered (triggering a biometric prompt for
 * registration). If a credential ID is found (either provided or in localStorage),
 * the existing passkey is used (triggering a biometric prompt for authentication).
 *
 * The returned `getKey` and `insertKey` callbacks are compatible with the
 * `ClientOptions.keystore` interface and transparently encrypt/decrypt keys
 * using the PRF-derived wrapping key.
 *
 * **Migration**: When `getKey` finds no encrypted entry in the keystore DB but
 * `storeName` is explicitly provided, it attempts to read the plaintext key from
 * the main client database's `accountAuths` table. If found, the key is
 * transparently re-encrypted and migrated to the keystore DB. Migration is only
 * available when `storeName` is explicitly provided (since the auto-generated DB
 * name is not known to JS before WASM initialization).
 *
 * @param {string} storeName - Store isolation key (from ClientOptions.storeName).
 * @param {object} [options]
 * @param {string} [options.credentialId] - Existing credential ID (base64url).
 * @param {string} [options.rpId] - WebAuthn relying party ID.
 * @param {string} [options.rpName] - Relying party display name.
 * @param {string} [options.userName] - User display name for the passkey.
 * @returns {Promise<{ getKey: Function, insertKey: Function, credentialId: string }>}
 */
export async function createPasskeyKeystore(storeName, options = {}) {
  if (!storeName || typeof storeName !== "string") {
    throw new Error("storeName is required and must be a non-empty string");
  }

  // Check browser support
  const supported = await isPasskeyPrfSupported();
  if (!supported) {
    throw new Error(
      "WebAuthn PRF extension is not supported in this browser. " +
        "Use isPasskeyPrfSupported() to check before enabling passkeyEncryption. " +
        "Supported browsers: Chrome 116+, Safari 18+, Edge 116+."
    );
  }

  // Resolve credential ID: provided > localStorage > register new
  let credentialId = options.credentialId || getStoredCredentialId(storeName);
  let prfOutput;

  if (credentialId) {
    // Authenticate with existing passkey
    prfOutput = await authenticateWithPrf(credentialId, options.rpId);
  } else {
    // Register new passkey
    const result = await registerPasskey({
      rpId: options.rpId,
      rpName: options.rpName,
      userName: options.userName,
    });
    credentialId = result.credentialId;
    prfOutput = result.prfOutput;
  }

  // Persist credential ID for future sessions
  storeCredentialId(storeName, credentialId);

  // Derive wrapping key (non-extractable, held in closure)
  const wrappingKey = await deriveWrappingKey(prfOutput);

  // Open encrypted keystore database
  const keystoreDb = openKeystoreDb(storeName);

  // Open main DB for migration (only when storeName is explicitly provided)
  const mainDb = openMainDbForMigration(storeName);

  /**
   * getKey callback — decrypts the secret key for a given pub key commitment.
   *
   * The `pubKey` parameter contains the pub key commitment bytes (RPO256 hash
   * of the public key), NOT the raw public key. This is a 32-byte Uint8Array.
   *
   * @param {Uint8Array} pubKey - Public key commitment bytes (32 bytes).
   * @returns {Promise<Uint8Array | undefined>}
   */
  async function getKey(pubKey) {
    const pubKeyHex = bytesToHex(pubKey);

    // Try encrypted keystore first
    const record = await keystoreDb.keys.get(pubKeyHex);
    if (record) {
      const ciphertext = hexToBytes(record.ciphertextHex);
      return await decryptSecretKey(wrappingKey, ciphertext, pubKey);
    }

    // Migration fallback: try main DB for plaintext keys
    if (mainDb) {
      try {
        const authRecord = await mainDb
          .table("accountAuths")
          .where("pubKeyCommitmentHex")
          .equals(pubKeyHex)
          .first();

        if (authRecord?.secretKeyHex) {
          const plaintext = hexToBytes(authRecord.secretKeyHex);

          // Re-encrypt and migrate to keystore DB
          const encrypted = await encryptSecretKey(
            wrappingKey,
            plaintext,
            pubKey
          );
          await keystoreDb.keys.put({
            pubKeyHex,
            ciphertextHex: bytesToHex(encrypted),
          });

          // Verify round-trip before deleting plaintext — if decryption
          // fails or bytes don't match, the plaintext entry is preserved.
          const verifyRecord = await keystoreDb.keys.get(pubKeyHex);
          const verifyCt = hexToBytes(verifyRecord.ciphertextHex);
          const decrypted = await decryptSecretKey(
            wrappingKey,
            verifyCt,
            pubKey
          );
          if (
            decrypted.length !== plaintext.length ||
            !decrypted.every((b, i) => b === plaintext[i])
          ) {
            throw new Error("Migration round-trip verification failed");
          }

          // Remove plaintext from old DB after successful verification
          try {
            await mainDb
              .table("accountAuths")
              .where("pubKeyCommitmentHex")
              .equals(pubKeyHex)
              .delete();
          } catch {
            // Best-effort cleanup — old DB schema may differ
          }

          return plaintext;
        }
      } catch (e) {
        // Main DB may not exist, have different schema, or be at a higher
        // Dexie version — migration is best-effort, not required.
        console.debug("Passkey migration: could not read from main DB", e);
      }
    }

    return undefined;
  }

  /**
   * insertKey callback — encrypts and stores the secret key.
   *
   * The `pubKey` parameter contains the pub key commitment bytes (RPO256 hash
   * of the public key), NOT the raw public key. This is a 32-byte Uint8Array.
   *
   * @param {Uint8Array} pubKey - Public key commitment bytes (32 bytes).
   * @param {Uint8Array} secretKey - Secret key bytes to encrypt.
   * @returns {Promise<void>}
   */
  async function insertKey(pubKey, secretKey) {
    const pubKeyHex = bytesToHex(pubKey);
    const encrypted = await encryptSecretKey(wrappingKey, secretKey, pubKey);

    // Upsert: put() overwrites existing entries. This is intentional —
    // the WASM side may re-insert during migration or key rotation.
    await keystoreDb.keys.put({
      pubKeyHex,
      ciphertextHex: bytesToHex(encrypted),
    });
  }

  return { getKey, insertKey, credentialId };
}
