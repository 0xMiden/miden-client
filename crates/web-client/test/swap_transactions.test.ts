// @ts-nocheck
import { test, expect } from "./test-setup";
import {
  setupWalletAndFaucet,
  mockMintAndConsume,
  mockSwap,
} from "./test-helpers";

// SWAP_TRANSACTION TEST
// =======================================================================================================

test.describe("swap transaction tests", () => {
  test("swap transaction completes successfully", async ({ client, sdk }) => {
    const { wallet: walletA, faucet: faucetA } = await setupWalletAndFaucet(
      client,
      sdk
    );
    const { wallet: walletB, faucet: faucetB } = await setupWalletAndFaucet(
      client,
      sdk
    );

    // Fund both accounts
    await mockMintAndConsume(client, sdk, walletA.id(), faucetA.id());
    await mockMintAndConsume(client, sdk, walletB.id(), faucetB.id());

    const { accountAAssets, accountBAssets } = await mockSwap(
      client,
      sdk,
      walletA.id(),
      walletB.id(),
      faucetA.id(),
      1,
      faucetB.id(),
      25,
      "private",
      "private"
    );

    // --- assertions for Account A ---
    const aA = accountAAssets.find(
      (a) => a.assetId === faucetA.id().toString()
    );
    expect(aA, `Expected to find faucetA asset on Account A`).toBeTruthy();
    expect(BigInt(aA.amount)).toEqual(999n);

    const aB = accountAAssets.find(
      (a) => a.assetId === faucetB.id().toString()
    );
    expect(aB, `Expected to find faucetB asset on Account A`).toBeTruthy();
    expect(BigInt(aB.amount)).toEqual(25n);

    // --- assertions for Account B ---
    const bA = accountBAssets.find(
      (a) => a.assetId === faucetA.id().toString()
    );
    expect(bA, `Expected to find faucetA asset on Account B`).toBeTruthy();
    expect(BigInt(bA.amount)).toEqual(1n);

    const bB = accountBAssets.find(
      (a) => a.assetId === faucetB.id().toString()
    );
    expect(bB, `Expected to find faucetB asset on Account B`).toBeTruthy();
    expect(BigInt(bB.amount)).toEqual(975n);
  });
});
