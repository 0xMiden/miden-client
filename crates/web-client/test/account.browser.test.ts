// Browser-only tests preserved from `next`.
import test from "./playwright.global.setup";
import { expect } from "@playwright/test";
import {
  StorageMode,
  createNewWallet,
  createNewFaucet,
  fundAccountFromFaucet,
} from "./webClientTestUtils";

test.describe("getAccountProof vault commitment (browser-only)", () => {
  test("returns vault details based on known_vault_commitment parameter", async ({
    page,
  }) => {
    // Create public wallet and faucet, then fund the wallet so it has assets on-chain
    const walletResult = await createNewWallet(page, {
      storageMode: StorageMode.PUBLIC,
      mutable: false,
      authSchemeId: 2,
    });
    const faucetResult = await createNewFaucet(page, StorageMode.PUBLIC);
    await fundAccountFromFaucet(page, walletResult.id, faucetResult.id);

    const proofResults = await page.evaluate(
      async ({ walletId, faucetId }) => {
        const endpoint = new window.Endpoint(window.rpcUrl);
        const rpcClient = new window.RpcClient(endpoint);
        const accountId = window.AccountId.fromHex(walletId);

        const emptyWord = new window.Word(new BigUint64Array([0n, 0n, 0n, 0n]));

        // Query 1: EMPTY_WORD — always fetches vault data (commitment never matches)
        const proof1 = await rpcClient.getAccountProof(
          accountId,
          undefined,
          undefined,
          emptyWord
        );
        const vaultCommitment = proof1.accountHeader()!.vaultCommitment();

        const vaultAssets = proof1.vaultFungibleAssets() ?? [];

        // Query 2: actual vault commitment — matches node state, should skip vault data
        // Note: passing vaultCommitment consumes the Word (wasm ownership transfer)
        const proof2 = await rpcClient.getAccountProof(
          accountId,
          undefined,
          undefined,
          vaultCommitment
        );

        // Query 3: undefined — vault data not requested
        const proof3 = await rpcClient.getAccountProof(accountId);

        return {
          numVaultAssetsQuery1: vaultAssets.length,
          vaultAssetFaucetId: vaultAssets[0]?.faucetId().toString() ?? null,
          vaultAssetAmount:
            vaultAssets[0] != null ? Number(vaultAssets[0].amount()) : null,
          numVaultAssetsQuery2: proof2.vaultFungibleAssets()?.length ?? null,
          numVaultAssetsQuery3: proof3.vaultFungibleAssets()?.length ?? null,
        };
      },
      { walletId: walletResult.id, faucetId: faucetResult.id }
    );

    // EMPTY_WORD — always fetches vault data with correct content
    expect(proofResults.numVaultAssetsQuery1).toBe(1);
    expect(proofResults.vaultAssetFaucetId).toBe(faucetResult.id);
    expect(proofResults.vaultAssetAmount).toBe(1000);
    // Actual vault commitment — matches, node skips vault data
    expect(proofResults.numVaultAssetsQuery2).toBe(0);
    // undefined — vault data not requested
    expect(proofResults.numVaultAssetsQuery3).toBe(0);
  });
});
