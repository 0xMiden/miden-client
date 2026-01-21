import test from "./playwright.global.setup";

import { expect } from "@playwright/test";

test.describe("remote keystore", () => {
  test("should create a client with a remote keystore", async ({ page }) => {
    const client = await page.evaluate(async () => {
      const insertKeyCb = async (
        _publicKeyCommitment: string,
        _secretKey: string
      ) => {};
      const getKeyCb = async (_publicKeyCommitment: string) => {
        return undefined;
      };
      const signCb = async (_publicKeyCommitment: string, _message: string) => {
        return undefined;
      };
      const client = await window.WebClient.createClientWithExternalKeystore(
        window.rpcUrl!,
        undefined,
        undefined,
        undefined,
        getKeyCb,
        insertKeyCb,
        signCb
      );
      return client;
    });
    expect(client).toBeDefined();
  });

  test("should create a client with a remote keystore and insert a key", async ({
    page,
  }) => {
    const { publicKeyCommitment, secretKey } = await page.evaluate(async () => {
      let publicKeyCommitment: string | undefined;
      let secretKey: string | undefined;
      const insertKeyCb = async (
        publicKeyCommitmentStr: string,
        secretKeyStr: string
      ) => {
        publicKeyCommitment = publicKeyCommitmentStr;
        secretKey = secretKeyStr;
      };
      const client = await window.WebClient.createClientWithExternalKeystore(
        window.rpcUrl!,
        undefined,
        undefined,
        undefined,
        undefined,
        insertKeyCb,
        undefined
      );
      await client.newWallet(
        window.AccountStorageMode.private(),
        true,
        0,
        undefined
      );

      return {
        publicKeyCommitment,
        secretKey,
      };
    });

    expect(publicKeyCommitment).toBeDefined();
    expect(secretKey).toBeDefined();
  });

  test("should call getKey callback with correct public key during export", async ({
    page,
  }) => {
    const { insertedPubKey, getKeyPubKey } = await page.evaluate(async () => {
      let insertedPubKey: number[] | undefined;
      let getKeyPubKey: number[] | undefined;

      const insertKeyCb = async (
        publicKeyCommitment: Uint8Array,
        _secretKey: Uint8Array
      ) => {
        insertedPubKey = Array.from(publicKeyCommitment);
      };

      const getKeyCb = async (publicKeyCommitment: Uint8Array) => {
        getKeyPubKey = Array.from(publicKeyCommitment);
        // Intentionally return undefined to cause export to fail after callback invocation
        return undefined;
      };

      const client = await window.WebClient.createClientWithExternalKeystore(
        window.rpcUrl!,
        undefined,
        undefined,
        undefined,
        getKeyCb,
        insertKeyCb,
        undefined
      );

      const wallet = await client.newWallet(
        window.AccountStorageMode.private(),
        true,
        0,
        undefined
      );

      try {
        await (client as any).exportAccountFile(wallet.id());
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
    const { faucetPubKey, signPubKey } = await page.evaluate(async () => {
      let faucetPubKey: number[] | undefined;
      let faucetSecretKey: Uint8Array | undefined;
      let signPubKey: number[] | undefined;

      const insertKeyCb = async (
        publicKeyCommitment: Uint8Array,
        secretKey: Uint8Array
      ) => {
        // Capture the faucet's public key (we will create the faucet first)
        if (!faucetPubKey) {
          faucetPubKey = Array.from(publicKeyCommitment);
          faucetSecretKey = secretKey;
        }
      };

      const signCb = async (
        publicKeyCommitment: Uint8Array,
        signingInputs: Uint8Array
      ) => {
        signPubKey = Array.from(publicKeyCommitment);
        const wasmSigningInputs =
          window.SigningInputs.deserialize(signingInputs);
        const wasmSecretKey = window.AuthSecretKey.deserialize(
          faucetSecretKey!
        );
        const signature = wasmSecretKey.signData(wasmSigningInputs);
        const serializedSig = signature.serialize();
        return serializedSig;
      };

      const client = await window.WebClient.createClientWithExternalKeystore(
        window.rpcUrl!,
        undefined,
        undefined,
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
        0,
        undefined
      );

      await client.syncState();

      const txRequest = (client as any).newMintTransactionRequest(
        wallet.id(),
        faucet.id(),
        window.NoteType.Public,
        BigInt(1000)
      );

      // This call should trigger the sign callback
      await client.executeTransaction(faucet.id(), txRequest);

      return { faucetPubKey, signPubKey };
    });

    expect(faucetPubKey).toBeDefined();
    expect(signPubKey).toBeDefined();
    expect(signPubKey).toEqual(faucetPubKey);
  });
});
