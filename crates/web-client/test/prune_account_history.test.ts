import test from "./playwright.global.setup";
import { expect } from "@playwright/test";
import { setupWalletAndFaucet, mintTransaction } from "./webClientTestUtils";

test.describe("prune_account_history tests", () => {
  test("prunes old committed states for a single account", async ({ page }) => {
    const { accountId, faucetId } = await setupWalletAndFaucet(page);

    // Mint twice — each mint advances the faucet nonce (0 → 1 → 2),
    // creating historical entries at each step.
    await mintTransaction(page, accountId, faucetId);
    await mintTransaction(page, accountId, faucetId);

    // Prune faucet history and verify the account is still intact
    const result = await page.evaluate(async (_faucetId: string) => {
      const client = window.client;
      const faucetAccountId = window.AccountId.fromHex(_faucetId);

      // Record state before pruning
      const accountBefore = await client.getAccount(faucetAccountId);
      const commitmentBefore = accountBefore!.to_commitment().toHex();

      // Prune
      const deleted = await client.pruneAccountHistory(faucetAccountId);

      // Verify account is still fully readable after pruning
      const accountAfter = await client.getAccount(faucetAccountId);
      const commitmentAfter = accountAfter!.to_commitment().toHex();

      return {
        deleted,
        commitmentBefore,
        commitmentAfter,
        nonce: accountAfter!.nonce().toString(),
        accountExists: accountAfter !== null && accountAfter !== undefined,
      };
    }, faucetId);

    expect(result.deleted).toBeGreaterThan(0);
    expect(result.accountExists).toBe(true);
    expect(result.commitmentBefore).toEqual(result.commitmentAfter);
    expect(Number(result.nonce)).toBeGreaterThanOrEqual(2);
  });

  test("prune is a no-op for accounts with a single state", async ({
    page,
  }) => {
    const result = await page.evaluate(async () => {
      const client = window.client;

      // Create a wallet but don't transact — it has only one historical state
      const wallet = await client.newWallet(
        window.AccountStorageMode.private(),
        true,
        window.AuthScheme.AuthRpoFalcon512
      );

      const deleted = await client.pruneAccountHistory(wallet.id());
      const accountAfter = await client.getAccount(wallet.id());

      return {
        deleted,
        accountExists: accountAfter !== null && accountAfter !== undefined,
      };
    });

    expect(result.deleted).toBe(0);
    expect(result.accountExists).toBe(true);
  });

  test("prunes all account history across multiple accounts", async ({
    page,
  }) => {
    const { accountId, faucetId } = await setupWalletAndFaucet(page);

    // Mint — advances faucet nonce, creating historical entries
    await mintTransaction(page, accountId, faucetId);
    await mintTransaction(page, accountId, faucetId);

    const result = await page.evaluate(
      async ({
        _accountId,
        _faucetId,
      }: {
        _accountId: string;
        _faucetId: string;
      }) => {
        const client = window.client;
        const walletId = window.AccountId.fromHex(_accountId);
        const faucetAccountId = window.AccountId.fromHex(_faucetId);

        const deleted = await client.pruneAllAccountHistory();

        // Both accounts should still be readable
        const wallet = await client.getAccount(walletId);
        const faucet = await client.getAccount(faucetAccountId);

        return {
          deleted,
          walletExists: wallet !== null && wallet !== undefined,
          faucetExists: faucet !== null && faucet !== undefined,
        };
      },
      { _accountId: accountId, _faucetId: faucetId }
    );

    expect(result.deleted).toBeGreaterThan(0);
    expect(result.walletExists).toBe(true);
    expect(result.faucetExists).toBe(true);
  });

  test("can send a transaction after pruning account history", async ({
    page,
  }) => {
    const { accountId, faucetId } = await setupWalletAndFaucet(page);

    // Mint twice to build history, then prune, then mint again to verify
    // the account is still fully functional post-pruning.
    await mintTransaction(page, accountId, faucetId);
    await mintTransaction(page, accountId, faucetId);

    // Prune faucet history
    await page.evaluate(async (_faucetId: string) => {
      const client = window.client;
      const faucetAccountId = window.AccountId.fromHex(_faucetId);
      await client.pruneAccountHistory(faucetAccountId);
    }, faucetId);

    // Mint again — this should succeed if pruning didn't break anything
    await mintTransaction(page, accountId, faucetId);

    const result = await page.evaluate(
      async ({ _faucetId }: { _faucetId: string }) => {
        const client = window.client;
        const faucetAccountId = window.AccountId.fromHex(_faucetId);
        const account = await client.getAccount(faucetAccountId);

        return {
          accountExists: account !== null && account !== undefined,
          nonce: account!.nonce().toString(),
        };
      },
      { _faucetId: faucetId }
    );

    expect(result.accountExists).toBe(true);
    expect(Number(result.nonce)).toBeGreaterThanOrEqual(3);
  });
});
