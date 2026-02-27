import {
  test as base,
  expect,
  BrowserContext,
  CDPSession,
} from "@playwright/test";
import { RUN_ID, getRpcUrl } from "./playwright.global.setup";

/**
 * Passkey keystore tests using Chrome DevTools Protocol virtual authenticator.
 *
 * These tests use CDP to create a virtual authenticator with PRF extension support,
 * allowing us to test the full passkey encryption flow without real hardware.
 *
 * Only runs on Chromium (CDP is not available on WebKit/Firefox).
 */

let cdpSession: CDPSession;
let authenticatorId: string;

function generateStoreName(suffix: string): string {
  return `test_passkey_${RUN_ID}_${suffix}`;
}

// Custom test fixture that sets up a virtual authenticator with PRF support
const test = base.extend<{ forEachTest: void }>({
  forEachTest: [
    async ({ page, browserName }, use) => {
      // Virtual authenticators with PRF require CDP (Chromium only)
      if (browserName !== "chromium") {
        test.skip();
        return;
      }

      cdpSession = await page.context().newCDPSession(page);

      // Enable the virtual authenticator environment
      await cdpSession.send("WebAuthn.enable", {
        enableUI: false,
      });

      // Add a virtual authenticator with PRF extension support
      const result = await cdpSession.send("WebAuthn.addVirtualAuthenticator", {
        options: {
          protocol: "ctap2",
          transport: "internal",
          hasResidentKey: true,
          hasUserVerification: true,
          isUserVerified: true,
          hasPrf: true,
        },
      });
      authenticatorId = result.authenticatorId;

      await page.goto("http://localhost:8080");

      // Import the SDK
      await page.evaluate(async () => {
        const sdkExports = await import("./index.js");
        for (const [key, value] of Object.entries(sdkExports)) {
          (window as any)[key] = value;
        }
      });

      await use();

      // Cleanup
      await cdpSession.send("WebAuthn.removeVirtualAuthenticator", {
        authenticatorId,
      });
      await cdpSession.send("WebAuthn.disable");
    },
    { auto: true },
  ],
});

test.describe("passkey keystore", () => {
  test("isPasskeyPrfSupported returns true with virtual authenticator", async ({
    page,
    browserName,
  }) => {
    test.skip(browserName !== "chromium", "CDP required");

    const supported = await page.evaluate(async () => {
      return await (window as any).isPasskeyPrfSupported();
    });

    expect(supported).toBe(true);
  });

  test("createPasskeyKeystore registers a new passkey and returns callbacks", async ({
    page,
    browserName,
  }) => {
    test.skip(browserName !== "chromium", "CDP required");

    const storeName = generateStoreName("register");

    const result = await page.evaluate(async (storeName) => {
      const { createPasskeyKeystore } = await import("./passkey-keystore.js");
      const keystore = await createPasskeyKeystore(storeName);
      return {
        hasGetKey: typeof keystore.getKey === "function",
        hasInsertKey: typeof keystore.insertKey === "function",
        hasCredentialId: typeof keystore.credentialId === "string",
        credentialIdLength: keystore.credentialId.length,
      };
    }, storeName);

    expect(result.hasGetKey).toBe(true);
    expect(result.hasInsertKey).toBe(true);
    expect(result.hasCredentialId).toBe(true);
    expect(result.credentialIdLength).toBeGreaterThan(0);
  });

  test("credential ID is persisted to localStorage and reused", async ({
    page,
    browserName,
  }) => {
    test.skip(browserName !== "chromium", "CDP required");

    const storeName = generateStoreName("persist");

    const { credentialId1, credentialId2 } = await page.evaluate(
      async (storeName) => {
        const { createPasskeyKeystore } = await import("./passkey-keystore.js");

        // First call — registers a new passkey
        const ks1 = await createPasskeyKeystore(storeName);
        const credentialId1 = ks1.credentialId;

        // Second call — should reuse the same credential from localStorage
        const ks2 = await createPasskeyKeystore(storeName);
        const credentialId2 = ks2.credentialId;

        return { credentialId1, credentialId2 };
      },
      storeName
    );

    expect(credentialId1).toBe(credentialId2);
  });

  test("insertKey encrypts and getKey decrypts round-trip", async ({
    page,
    browserName,
  }) => {
    test.skip(browserName !== "chromium", "CDP required");

    const storeName = generateStoreName("roundtrip");

    const result = await page.evaluate(async (storeName) => {
      const { createPasskeyKeystore } = await import("./passkey-keystore.js");
      const keystore = await createPasskeyKeystore(storeName);

      // Create fake pub key commitment (32 bytes) and secret key
      const pubKey = new Uint8Array(32);
      crypto.getRandomValues(pubKey);
      const secretKey = new Uint8Array(64);
      crypto.getRandomValues(secretKey);

      // Insert (encrypts)
      await keystore.insertKey(pubKey, secretKey);

      // Get (decrypts)
      const retrieved = await keystore.getKey(pubKey);

      return {
        original: Array.from(secretKey),
        retrieved: retrieved ? Array.from(retrieved) : null,
      };
    }, storeName);

    expect(result.retrieved).toEqual(result.original);
  });

  test("getKey returns undefined for unknown key", async ({
    page,
    browserName,
  }) => {
    test.skip(browserName !== "chromium", "CDP required");

    const storeName = generateStoreName("unknown");

    const result = await page.evaluate(async (storeName) => {
      const { createPasskeyKeystore } = await import("./passkey-keystore.js");
      const keystore = await createPasskeyKeystore(storeName);

      const unknownPubKey = new Uint8Array(32);
      crypto.getRandomValues(unknownPubKey);
      return await keystore.getKey(unknownPubKey);
    }, storeName);

    expect(result).toBeUndefined();
  });

  test("encrypted data in IndexedDB starts with MWEB magic", async ({
    page,
    browserName,
  }) => {
    test.skip(browserName !== "chromium", "CDP required");

    const storeName = generateStoreName("magic");

    const magicBytes = await page.evaluate(async (storeName) => {
      const { createPasskeyKeystore } = await import("./passkey-keystore.js");
      const keystore = await createPasskeyKeystore(storeName);

      const pubKey = new Uint8Array(32);
      crypto.getRandomValues(pubKey);
      const secretKey = new Uint8Array(64);
      crypto.getRandomValues(secretKey);

      await keystore.insertKey(pubKey, secretKey);

      // Read raw data from IndexedDB to verify MWEB magic
      const pubKeyHex = Array.from(pubKey)
        .map((b: number) => b.toString(16).padStart(2, "0"))
        .join("");
      const record = await new Promise<any>((resolve, reject) => {
        const req = indexedDB.open(`MidenKeystore_${storeName}`);
        req.onsuccess = () => {
          const tx = req.result.transaction("keys", "readonly");
          const getReq = tx.objectStore("keys").get(pubKeyHex);
          getReq.onsuccess = () => resolve(getReq.result);
          getReq.onerror = () => reject(getReq.error);
        };
        req.onerror = () => reject(req.error);
      });
      const ciphertextHex = record?.ciphertextHex as string;

      // First 4 bytes (8 hex chars) should be "MWEB" = 4d574542
      return ciphertextHex?.substring(0, 8);
    }, storeName);

    expect(magicBytes).toBe("4d574542");
  });

  test("different pub keys produce different ciphertexts", async ({
    page,
    browserName,
  }) => {
    test.skip(browserName !== "chromium", "CDP required");

    const storeName = generateStoreName("unique-ct");

    const result = await page.evaluate(async (storeName) => {
      const { createPasskeyKeystore } = await import("./passkey-keystore.js");
      const keystore = await createPasskeyKeystore(storeName);

      const secretKey = new Uint8Array(64);
      crypto.getRandomValues(secretKey);

      const pubKey1 = new Uint8Array(32);
      crypto.getRandomValues(pubKey1);
      const pubKey2 = new Uint8Array(32);
      crypto.getRandomValues(pubKey2);

      await keystore.insertKey(pubKey1, secretKey);
      await keystore.insertKey(pubKey2, secretKey);

      // Read raw ciphertexts from IndexedDB
      const hex1 = Array.from(pubKey1)
        .map((b: number) => b.toString(16).padStart(2, "0"))
        .join("");
      const hex2 = Array.from(pubKey2)
        .map((b: number) => b.toString(16).padStart(2, "0"))
        .join("");

      const readRecord = (key: string) =>
        new Promise<any>((resolve, reject) => {
          const req = indexedDB.open(`MidenKeystore_${storeName}`);
          req.onsuccess = () => {
            const tx = req.result.transaction("keys", "readonly");
            const getReq = tx.objectStore("keys").get(key);
            getReq.onsuccess = () => resolve(getReq.result);
            getReq.onerror = () => reject(getReq.error);
          };
          req.onerror = () => reject(req.error);
        });

      const record1 = await readRecord(hex1);
      const record2 = await readRecord(hex2);

      return {
        ct1: record1?.ciphertextHex,
        ct2: record2?.ciphertextHex,
      };
    }, storeName);

    // Same plaintext with different AAD (pub key commitment) → different ciphertext
    // Also, random IV ensures uniqueness
    expect(result.ct1).not.toBe(result.ct2);
  });

  test("ciphertext bound to pub key commitment (AAD prevents swapping)", async ({
    page,
    browserName,
  }) => {
    test.skip(browserName !== "chromium", "CDP required");

    const storeName = generateStoreName("aad");

    const decryptFailed = await page.evaluate(async (storeName) => {
      const { createPasskeyKeystore } = await import("./passkey-keystore.js");
      const keystore = await createPasskeyKeystore(storeName);

      const pubKey1 = new Uint8Array(32);
      crypto.getRandomValues(pubKey1);
      const pubKey2 = new Uint8Array(32);
      crypto.getRandomValues(pubKey2);
      const secretKey = new Uint8Array(64);
      crypto.getRandomValues(secretKey);

      await keystore.insertKey(pubKey1, secretKey);

      // Manually swap the ciphertext to pubKey2's entry in IndexedDB
      const hex1 = Array.from(pubKey1)
        .map((b: number) => b.toString(16).padStart(2, "0"))
        .join("");
      const hex2 = Array.from(pubKey2)
        .map((b: number) => b.toString(16).padStart(2, "0"))
        .join("");

      const dbName = `MidenKeystore_${storeName}`;
      const record = await new Promise<any>((resolve, reject) => {
        const req = indexedDB.open(dbName);
        req.onsuccess = () => {
          const tx = req.result.transaction("keys", "readonly");
          const getReq = tx.objectStore("keys").get(hex1);
          getReq.onsuccess = () => resolve(getReq.result);
          getReq.onerror = () => reject(getReq.error);
        };
        req.onerror = () => reject(req.error);
      });
      // Put the same ciphertext under pubKey2
      await new Promise<void>((resolve, reject) => {
        const req = indexedDB.open(dbName);
        req.onsuccess = () => {
          const tx = req.result.transaction("keys", "readwrite");
          const putReq = tx.objectStore("keys").put({
            pubKeyHex: hex2,
            ciphertextHex: record?.ciphertextHex,
          });
          putReq.onsuccess = () => resolve();
          putReq.onerror = () => reject(putReq.error);
        };
        req.onerror = () => reject(req.error);
      });

      // Attempting to decrypt with pubKey2 should fail (AAD mismatch)
      try {
        await keystore.getKey(pubKey2);
        return false; // Should not reach here
      } catch {
        return true; // AES-GCM auth tag verification failed
      }
    }, storeName);

    expect(decryptFailed).toBe(true);
  });

  test("works with MidenClient.create via passkeyEncryption option", async ({
    page,
    browserName,
  }) => {
    test.skip(browserName !== "chromium", "CDP required");

    const storeName = generateStoreName("client-api");
    const rpcUrl = getRpcUrl();

    const result = await page.evaluate(
      async ({ storeName, rpcUrl }) => {
        const MidenClient = (window as any).MidenClient;
        const client = await MidenClient.create({
          rpcUrl,
          storeName,
          passkeyEncryption: true,
        });
        return {
          clientCreated: client != null,
          hasAccounts: typeof client.accounts === "object",
          hasTransactions: typeof client.transactions === "object",
        };
      },
      { storeName, rpcUrl }
    );

    expect(result.clientCreated).toBe(true);
    expect(result.hasAccounts).toBe(true);
    expect(result.hasTransactions).toBe(true);
  });

  test("explicit credentialId option reuses existing passkey", async ({
    page,
    browserName,
  }) => {
    test.skip(browserName !== "chromium", "CDP required");

    const storeName = generateStoreName("explicit-cred");

    const result = await page.evaluate(async (storeName) => {
      const { createPasskeyKeystore } = await import("./passkey-keystore.js");

      // Register
      const ks1 = await createPasskeyKeystore(storeName);
      const credentialId = ks1.credentialId;

      // Clear localStorage to prove the explicit option works
      localStorage.removeItem(`miden_passkey_credential_${storeName}`);

      // Re-create with explicit credentialId
      const differentStore = storeName + "_reuse";
      const ks2 = await createPasskeyKeystore(differentStore, {
        credentialId,
      });

      // Insert with ks1, retrieve with ks2 (same PRF → same wrapping key)
      const pubKey = new Uint8Array(32);
      crypto.getRandomValues(pubKey);
      const secretKey = new Uint8Array(48);
      crypto.getRandomValues(secretKey);

      await ks1.insertKey(pubKey, secretKey);

      // ks2 has the same wrapping key but different DB
      // So we need to use the same DB by using the same storeName
      const ks3 = await createPasskeyKeystore(storeName, { credentialId });
      const retrieved = await ks3.getKey(pubKey);

      return {
        credentialId1: credentialId,
        credentialId2: ks2.credentialId,
        original: Array.from(secretKey),
        retrieved: retrieved ? Array.from(retrieved) : null,
      };
    }, storeName);

    expect(result.credentialId1).toBe(result.credentialId2);
    expect(result.retrieved).toEqual(result.original);
  });
});
