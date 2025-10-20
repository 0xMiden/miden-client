import test from "./playwright.global.setup";
import { Page, expect } from "@playwright/test";
import {
  clearStore,
  createNewFaucet,
  createNewWallet,
  fundAccountFromFaucet,
  getAccount,
  getAccountBalance,
  StorageMode,
} from "./webClientTestUtils";

const importWalletFromSeed = async (
  page: Page,
  walletSeed: Uint8Array,
  mutable: boolean
) => {
  return await page.evaluate(
    async ({ _walletSeed, mutable }) => {
      const client = window.client;
      await client.syncState();
      const account = await client.importPublicAccountFromSeed(
        _walletSeed,
        mutable
      );
      return {
        accountId: account.id().toString(),
        accountCommitment: account.commitment().toHex(),
      };
    },
    {
      _walletSeed: walletSeed,
      mutable,
    }
  );
};

const importAccountById = async (page: Page, accountId: string) => {
  return await page.evaluate(async (id) => {
    const client = window.client;
    const _accountId = window.AccountId.fromHex(id);
    await client.importAccountById(_accountId);
    const account = await client.getAccount(_accountId);
    return {
      accountId: account?.id().toString(),
      accountCommitment: account?.commitment().toHex(),
    };
  }, accountId);
};

test.describe("import from seed", () => {
  // Disabled due to out of bounds memory error:
  // https://github.com/0xMiden/miden-vm/issues/2303
  test.skip("should import same public account from seed", async ({ page }) => {
    const walletSeed = new Uint8Array(32);
    crypto.getRandomValues(walletSeed);

    const mutable = false;
    const storageMode = StorageMode.PUBLIC;

    const initialWallet = await createNewWallet(page, {
      storageMode,
      mutable,
      walletSeed,
    });

    const faucet = await createNewFaucet(page);

    const result = await fundAccountFromFaucet(
      page,
      initialWallet.id,
      faucet.id
    );
    const initialBalance = result.targetAccountBalance;

    const { commitment: initialCommitment } = await getAccount(
      page,
      initialWallet.id
    );

    // Deleting the account
    await clearStore(page);

    const { accountId: restoredAccountId } = await importWalletFromSeed(
      page,
      walletSeed,
      mutable
    );

    expect(restoredAccountId).toEqual(initialWallet.id);

    const { commitment: restoredAccountCommitment } = await getAccount(
      page,
      initialWallet.id
    );

    const restoredBalance = await getAccountBalance(
      page,
      initialWallet.id,
      faucet.id
    );

    expect(restoredBalance!.toString()).toEqual(initialBalance);
    expect(restoredAccountCommitment).toEqual(initialCommitment);
  });
});

test.describe("import public account by id", () => {
  // Disabled due to out of bounds memory error:
  // https://github.com/0xMiden/miden-vm/issues/2303
  test.skip("should import public account from id", async ({ page }) => {
    const walletSeed = new Uint8Array(32);
    crypto.getRandomValues(walletSeed);

    const mutable = false;
    const storageMode = StorageMode.PUBLIC;

    const initialWallet = await createNewWallet(page, {
      storageMode,
      mutable,
      walletSeed,
    });
    const faucet = await createNewFaucet(page);
    const { targetAccountBalance: initialBalance } =
      await fundAccountFromFaucet(page, initialWallet.id, faucet.id);
    const { commitment: initialCommitment } = await getAccount(
      page,
      initialWallet.id
    );

    await clearStore(page);

    const { accountId: restoredAccountId } = await importAccountById(
      page,
      initialWallet.id
    );
    expect(restoredAccountId).toEqual(initialWallet.id);

    const { commitment: restoredAccountCommitment } = await getAccount(
      page,
      initialWallet.id
    );
    const restoredBalance = await getAccountBalance(
      page,
      initialWallet.id,
      faucet.id
    );

    expect(restoredBalance!.toString()).toEqual(initialBalance);
    expect(restoredAccountCommitment).toEqual(initialCommitment);
  });
});
