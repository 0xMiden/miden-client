// @ts-nocheck
import { test, expect } from "./test-setup";

test.describe("prune_account_history tests", () => {
  test("prunes old committed states for a single account", async ({ run }) => {
    test.slow();
    const result = await run(async ({ client, sdk, helpers }) => {
      const { wallet, faucet } = await helpers.setupWalletAndFaucet();

      // Mint twice: each mint advances the faucet nonce (0 to 1 to 2),
      // creating historical entries at each step.
      await helpers.mockMintAndConsume(wallet.id(), faucet.id());
      await helpers.mockMintAndConsume(wallet.id(), faucet.id());

      // Record state before pruning
      const accountBefore = await client.getAccount(faucet.id());
      const commitmentBefore = accountBefore.to_commitment().toHex();

      // Prune faucet history up to nonce 1
      const deleted = await client.pruneAccountHistory(
        faucet.id(),
        new sdk.Felt(1n)
      );

      // Verify account is still fully readable after pruning
      const accountAfter = await client.getAccount(faucet.id());
      const commitmentAfter = accountAfter.to_commitment().toHex();

      return {
        deleted,
        commitmentBefore,
        commitmentAfter,
        nonce: accountAfter.nonce().toString(),
        accountExists: accountAfter !== undefined,
      };
    });

    expect(result.deleted).toBeGreaterThan(0);
    expect(result.accountExists).toBe(true);
    expect(result.commitmentBefore).toEqual(result.commitmentAfter);
    expect(Number(result.nonce)).toBeGreaterThanOrEqual(2);
  });

  test("prune is a no-op when nonce is 0", async ({ run }) => {
    const result = await run(async ({ client, sdk }) => {
      // Create a wallet but don't transact: it has only one historical state
      const wallet = await client.newWallet(
        sdk.AccountStorageMode.private(),
        true,
        sdk.AuthScheme.AuthRpoFalcon512
      );

      // Prune with nonce 0: nothing should be deleted
      const deleted = await client.pruneAccountHistory(
        wallet.id(),
        new sdk.Felt(0n)
      );
      const accountAfter = await client.getAccount(wallet.id());

      return {
        deleted,
        accountExists: accountAfter !== undefined,
      };
    });

    expect(result.deleted).toBe(0);
    expect(result.accountExists).toBe(true);
  });

  test("can send a transaction after pruning account history", async ({
    run,
  }) => {
    test.slow();
    const result = await run(async ({ client, sdk, helpers }) => {
      const { wallet, faucet } = await helpers.setupWalletAndFaucet();

      // Mint twice to build history, then prune, then mint again to verify
      // the account is still fully functional post-pruning.
      await helpers.mockMintAndConsume(wallet.id(), faucet.id());
      await helpers.mockMintAndConsume(wallet.id(), faucet.id());

      // Prune faucet history up to nonce 1
      await client.pruneAccountHistory(faucet.id(), new sdk.Felt(1n));

      // Mint again: this should succeed if pruning didn't break anything
      await helpers.mockMintAndConsume(wallet.id(), faucet.id());

      const account = await client.getAccount(faucet.id());

      return {
        accountExists: account !== undefined,
        nonce: account.nonce().toString(),
      };
    });

    expect(result.accountExists).toBe(true);
    expect(Number(result.nonce)).toBeGreaterThanOrEqual(3);
  });
});
