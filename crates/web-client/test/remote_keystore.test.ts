import test from "./playwright.global.setup";

import { expect } from "@playwright/test";

test.describe.only("remote keystore", () => {
  test("should create a client with a remote keystore", async ({ page }) => {
    const client = await page.evaluate(async () => {
      const insertKeyCb = async (_publicKey: string, _secretKey: string) => {};
      const getKeyCb = async (_publicKey: string) => {
        return undefined;
      };
      const signCb = async (_publicKey: string, _message: string) => {
        return undefined;
      };
      const client = await window.WebClient.createClientWithExternalKeystore(
        window.rpcUrl,
        undefined,
        insertKeyCb,
        getKeyCb,
        signCb
      );
      return client;
    });
    console.log("client", client);
    expect(client).toBeDefined();
  });

  test("should create a client with a remote keystore and insert a key", async ({
    page,
  }) => {
    const { publicKey, secretKey, publicKeySerialized, secretKeySerialized } =
      await page.evaluate(async () => {
        let publicKey: string | undefined;
        let secretKey: string | undefined;
        const insertKeyCb = async (
          publicKeyStr: string,
          secretKeyStr: string
        ) => {
          publicKey = publicKeyStr;
          secretKey = secretKeyStr;
        };
        const client = await window.WebClient.createClientWithExternalKeystore(
          window.rpcUrl,
          undefined,
          undefined,
          insertKeyCb,
          undefined
        );
        await client.newWallet(
          window.AccountStorageMode.private(),
          true,
          undefined
        );

        return {
          publicKey,
          secretKey,
        };
      });
    console.log("publicKey", publicKey);
    console.log("secretKey", secretKey);
    expect(publicKey).toBeDefined();
    expect(secretKey).toBeDefined();
  });
});
