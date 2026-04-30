// @ts-nocheck
import { test, expect } from "./test-setup";

// SWAP_TRANSACTION TEST
// =======================================================================================================

test.describe("swap transaction tests", () => {
  test("swap transaction completes successfully", async ({ run }) => {
    const result = await run(async ({ client, sdk, helpers }) => {
      const { wallet: walletA, faucet: faucetA } =
        await helpers.setupWalletAndFaucet();
      const { wallet: walletB, faucet: faucetB } =
        await helpers.setupWalletAndFaucet();

      // Fund both accounts
      await helpers.mockMintAndConsume(walletA.id(), faucetA.id());
      await helpers.mockMintAndConsume(walletB.id(), faucetB.id());

      const faucetAId = faucetA.id().toString();
      const faucetBId = faucetB.id().toString();

      const { accountAAssets, accountBAssets } = await helpers.mockSwap(
        walletA.id(),
        walletB.id(),
        faucetA.id(),
        1,
        faucetB.id(),
        25,
        "private",
        "private"
      );

      return { accountAAssets, accountBAssets, faucetAId, faucetBId };
    });

    // --- assertions for Account A ---
    const aA = result.accountAAssets.find(
      (a) => a.assetId === result.faucetAId
    );
    expect(aA, `Expected to find faucetA asset on Account A`).toBeTruthy();
    expect(BigInt(aA.amount)).toEqual(999n);

    const aB = result.accountAAssets.find(
      (a) => a.assetId === result.faucetBId
    );
    expect(aB, `Expected to find faucetB asset on Account A`).toBeTruthy();
    expect(BigInt(aB.amount)).toEqual(25n);

    // --- assertions for Account B ---
    const bA = result.accountBAssets.find(
      (a) => a.assetId === result.faucetAId
    );
    expect(bA, `Expected to find faucetA asset on Account B`).toBeTruthy();
    expect(BigInt(bA.amount)).toEqual(1n);

    const bB = result.accountBAssets.find(
      (a) => a.assetId === result.faucetBId
    );
    expect(bB, `Expected to find faucetB asset on Account B`).toBeTruthy();
    expect(BigInt(bB.amount)).toEqual(975n);
  });
});
