import test from "./playwright.global.setup";
import { expect } from "@playwright/test";
import {
  mintAndConsumeTransaction,
  setupWalletAndFaucet,
  swapTransaction,
} from "./webClientTestUtils";

// SWAP_TRANSACTION TEST
// =======================================================================================================

test.describe("swap transaction tests", () => {
  test("swap transaction completes successfully", async ({ page }) => {
    const { accountId: accountA, faucetId: faucetA } =
      await setupWalletAndFaucet(page);
    const { accountId: accountB, faucetId: faucetB } =
      await setupWalletAndFaucet(page);

    const assetAAmount = BigInt(1);
    const assetBAmount = BigInt(25);

    // Mint/consume once and reuse the setup for both prover modes.
    await mintAndConsumeTransaction(page, accountA, faucetA, false);
    await mintAndConsumeTransaction(page, accountB, faucetB, false);

    const runSwapAssertions = async (withRemoteProver: boolean) => {
      const { accountAAssets, accountBAssets } = await swapTransaction(
        page,
        accountA,
        accountB,
        faucetA,
        assetAAmount,
        faucetB,
        assetBAmount,
        "private",
        "private",
        withRemoteProver
      );

      // --- assertions for Account A ---
      const aA = accountAAssets!.find((a) => a.assetId === faucetA);
      expect(aA, `Expected to find asset ${faucetA} on Account A`).toBeTruthy();
      expect(BigInt(aA!.amount)).toEqual(999n);

      const aB = accountAAssets!.find((a) => a.assetId === faucetB);
      expect(aB, `Expected to find asset ${faucetB} on Account A`).toBeTruthy();
      expect(BigInt(aB!.amount)).toEqual(25n);

      // --- assertions for Account B ---
      const bA = accountBAssets!.find((a) => a.assetId === faucetA);
      expect(bA, `Expected to find asset ${faucetA} on Account B`).toBeTruthy();
      expect(BigInt(bA!.amount)).toEqual(1n);

      const bB = accountBAssets!.find((a) => a.assetId === faucetB);
      expect(bB, `Expected to find asset ${faucetB} on Account B`).toBeTruthy();
      expect(BigInt(bB!.amount)).toEqual(975n);
    };

    await runSwapAssertions(false);
    await runSwapAssertions(true);
  });
});
