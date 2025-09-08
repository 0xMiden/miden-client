import test from "./playwright.global.setup";

import { expect } from "@playwright/test";

test.describe("remote keystore", () => {
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
    expect(client).toBeDefined();
  });

  test("should create a client with a remote keystore and insert a key", async ({
    page,
  }) => {
    const { publicKey, secretKey } = await page.evaluate(async () => {
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

    expect(publicKey).toBeDefined();
    expect(secretKey).toBeDefined();
  });

  test("should call getKey callback with correct public key during export", async ({
    page,
  }) => {
    const { insertedPubKey, getKeyPubKey } = await page.evaluate(async () => {
      let insertedPubKey: number[] | undefined;
      let getKeyPubKey: number[] | undefined;

      const insertKeyCb = async (
        publicKey: Uint8Array,
        _secretKey: Uint8Array
      ) => {
        insertedPubKey = Array.from(publicKey);
      };

      const getKeyCb = async (publicKey: Uint8Array) => {
        getKeyPubKey = Array.from(publicKey);
        // Intentionally return undefined to cause export to fail after callback invocation
        return undefined;
      };

      const client = await window.WebClient.createClientWithExternalKeystore(
        window.rpcUrl,
        undefined,
        getKeyCb,
        insertKeyCb,
        undefined
      );

      const wallet = await client.newWallet(
        window.AccountStorageMode.private(),
        true,
        undefined
      );

      try {
        const anyClient: any = client as any;
        await anyClient.exportAccountFile(wallet.id());
      } catch (_e) {
        // Expected due to undefined return from getKeyCb; we only care that the callback was invoked
      }

      return { insertedPubKey, getKeyPubKey };
    });

    expect(insertedPubKey).toBeDefined();
    expect(getKeyPubKey).toBeDefined();
    expect(getKeyPubKey).toEqual(insertedPubKey);
  });

  test("should call sign callback with correct arguments during transaction", async ({
    page,
  }) => {
    const { faucetPubKey, signPubKey, commitmentLength, pubKeyLength } =
      await page.evaluate(async () => {
        let faucetPubKey: number[] | undefined;
        let signPubKey: number[] | undefined;
        let commitmentLength: number | undefined;
        let pubKeyLength: number | undefined;

        const insertKeyCb = async (
          publicKey: Uint8Array,
          _secretKey: Uint8Array
        ) => {
          // Capture the faucet's public key (we will create the faucet first)
          if (!faucetPubKey) {
            faucetPubKey = Array.from(publicKey);
          }
        };

        const signCb = async (
          publicKey: Uint8Array,
          commitment: Uint8Array
        ) => {
          signPubKey = Array.from(publicKey);
          commitmentLength = commitment.byteLength;
          pubKeyLength = publicKey.byteLength;
          // Return a trivially parseable signature (numeric strings)
          return ["1", "2", "3", "4"];
        };

        const client = await window.WebClient.createClientWithExternalKeystore(
          window.rpcUrl,
          undefined,
          undefined,
          insertKeyCb,
          signCb
        );

        // Create faucet first so insertKeyCb captures its public key
        const faucet = await client.newFaucet(
          window.AccountStorageMode.private(),
          false,
          "DAG",
          8,
          BigInt(10000000)
        );

        await client.syncState();

        const wallet = await client.newWallet(
          window.AccountStorageMode.private(),
          true,
          undefined
        );

        await client.syncState();

        const anyClient: any = client as any;
        const txRequest = anyClient.newMintTransactionRequest(
          wallet.id(),
          faucet.id(),
          window.NoteType.Private,
          BigInt(1000)
        );

        // This call should trigger the sign callback
        await client.newTransaction(faucet.id(), txRequest);

        return { faucetPubKey, signPubKey, commitmentLength, pubKeyLength };
      });

    expect(faucetPubKey).toBeDefined();
    expect(signPubKey).toBeDefined();
    expect(signPubKey).toEqual(faucetPubKey);
    expect(commitmentLength).toBe(32);
    expect(pubKeyLength).toBe(32);
  });
});
