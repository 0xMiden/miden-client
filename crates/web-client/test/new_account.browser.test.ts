// Browser-only tests preserved from `next`.
// These require `isolatedClient` helper which only works in the browser.
import test from "./playwright.global.setup";
import { expect } from "@playwright/test";
import { createNewWallet, StorageMode } from "./webClientTestUtils";

test.describe("new_wallet tests (browser-only)", () => {
  test("Constructs the same account when given the same init seed", async ({
    page,
  }) => {
    const clientSeed1 = new Uint8Array(32);
    const clientSeed2 = new Uint8Array(32);
    const walletSeed = new Uint8Array(32);
    crypto.getRandomValues(clientSeed1);
    crypto.getRandomValues(clientSeed2);
    crypto.getRandomValues(walletSeed);

    // Isolate the client instance both times to ensure the outcome is deterministic
    await createNewWallet(page, {
      storageMode: StorageMode.PUBLIC,
      mutable: false,
      authSchemeId: 2,
      clientSeed: clientSeed1,
      isolatedClient: true,
      walletSeed: walletSeed,
    });

    // This should fail, as the wallet is already tracked within the same browser context
    await expect(async () => {
      await createNewWallet(page, {
        storageMode: StorageMode.PUBLIC,
        mutable: false,
        authSchemeId: 2,
        clientSeed: clientSeed2,
        isolatedClient: true,
        walletSeed: walletSeed,
      });
    }).rejects.toThrowError(/failed to insert new wallet/);
  });
});
