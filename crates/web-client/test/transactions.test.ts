// FIXME: Re-visit this
// @ts-nocheck
import test from "./playwright.global.setup";
import {
  consumeTransaction,
  mintAndConsumeTransaction,
  mintTransaction,
  setupWalletAndFaucet,
} from "./webClientTestUtils";
import { Page, expect } from "@playwright/test";

// GET_TRANSACTIONS TESTS
// =======================================================================================================

interface GetAllTransactionsResult {
  transactionIds: string[];
  uncommittedTransactionIds: string[];
}

const getAllTransactions = async (
  page: Page
): Promise<GetAllTransactionsResult> => {
  return await page.evaluate(async () => {
    const client = window.client;

    let transactions = await client.getTransactions(
      window.TransactionFilter.all()
    );
    let uncommittedTransactions = await client.getTransactions(
      window.TransactionFilter.uncommitted()
    );
    let transactionIds = transactions.map((transaction) =>
      transaction.id().toHex()
    );
    let uncommittedTransactionIds = uncommittedTransactions.map((transaction) =>
      transaction.id().toHex()
    );

    return {
      transactionIds: transactionIds,
      uncommittedTransactionIds: uncommittedTransactionIds,
    };
  });
};

test.describe("get_transactions tests", () => {
  test("get_transactions retrieves all transactions successfully", async ({
    page,
  }) => {
    const { accountId, faucetId } = await setupWalletAndFaucet(page);

    const { mintResult, consumeResult } = await mintAndConsumeTransaction(
      page,
      accountId,
      faucetId
    );

    const result = await getAllTransactions(page);

    expect(result.transactionIds).toContain(mintResult.transactionId);
    expect(result.transactionIds).toContain(consumeResult.transactionId);
    expect(result.uncommittedTransactionIds.length).toEqual(0);
  });

  test("get_transactions retrieves uncommitted transactions successfully", async ({
    page,
  }) => {
    const { accountId, faucetId } = await setupWalletAndFaucet(page);
    const { mintResult, consumeResult } = await mintAndConsumeTransaction(
      page,
      accountId,
      faucetId
    );
    const { transactionId: uncommittedTransactionId } = await mintTransaction(
      page,
      accountId,
      faucetId,
      false,
      false
    );

    const result = await getAllTransactions(page);

    expect(result.transactionIds).toContain(mintResult.transactionId);
    expect(result.transactionIds).toContain(consumeResult.transactionId);
    expect(result.transactionIds).toContain(uncommittedTransactionId);
    expect(result.transactionIds.length).toEqual(3);

    expect(result.uncommittedTransactionIds).toContain(
      uncommittedTransactionId
    );
    expect(result.uncommittedTransactionIds.length).toEqual(1);
  });

  test("get_transactions retrieves no transactions successfully", async ({
    page,
  }) => {
    const result = await getAllTransactions(page);

    expect(result.transactionIds.length).toEqual(0);
    expect(result.uncommittedTransactionIds.length).toEqual(0);
  });

  test("get_transactions filters by specific transaction IDs successfully", async ({
    page,
  }) => {
    const { accountId, faucetId } = await setupWalletAndFaucet(page);

    await mintAndConsumeTransaction(page, accountId, faucetId);

    const result = await page.evaluate(async () => {
      const client = window.client;

      let allTransactions = await client.getTransactions(
        window.TransactionFilter.all()
      );
      const allTxLength = allTransactions.length;
      let firstTransactionId = allTransactions[0].id();
      const firstTxIdHex = firstTransactionId.toHex();

      const filter = window.TransactionFilter.ids([firstTransactionId]);
      let filteredTransactions = await client.getTransactions(filter);
      const filteredTransactionIds = filteredTransactions.map((tx) =>
        tx.id().toHex()
      );

      return {
        allTransactionsCount: allTxLength,
        filteredTransactionIds: filteredTransactionIds,
        originalTransactionId: firstTxIdHex,
      };
    });

    expect(result.allTransactionsCount).toEqual(2);
    expect(result.filteredTransactionIds.length).toEqual(1);
    expect(result.filteredTransactionIds).toContain(
      result.originalTransactionId
    );
  });

  test("get_transactions filters expired transactions successfully", async ({
    page,
  }) => {
    const { accountId, faucetId } = await setupWalletAndFaucet(page);

    const { transactionId: committedTransactionId } = await mintTransaction(
      page,
      accountId,
      faucetId
    );

    const { transactionId: uncommittedTransactionId } = await mintTransaction(
      page,
      accountId,
      faucetId,
      false,
      false
    );

    const result = await page.evaluate(async () => {
      const client = window.client;

      let allTransactions = await client.getTransactions(
        window.TransactionFilter.all()
      );
      let allTransactionIds = allTransactions.map((tx) => tx.id().toHex());
      let currentBlockNum = allTransactions[0].blockNum();

      let futureBlockNum = currentBlockNum + 10;
      let futureExpiredFilter =
        window.TransactionFilter.expiredBefore(futureBlockNum);
      let futureExpiredTransactions =
        await client.getTransactions(futureExpiredFilter);
      let futureExpiredTransactionIds = futureExpiredTransactions.map((tx) =>
        tx.id().toHex()
      );

      let pastBlockNum = currentBlockNum - 10;
      let pastExpiredFilter =
        window.TransactionFilter.expiredBefore(pastBlockNum);
      let pastExpiredTransactions =
        await client.getTransactions(pastExpiredFilter);
      let pastExpiredTransactionIds = pastExpiredTransactions.map((tx) =>
        tx.id().toHex()
      );

      return {
        currentBlockNum: currentBlockNum,
        futureBlockNum: futureBlockNum,
        pastBlockNum: pastBlockNum,
        allTransactionIds: allTransactionIds,
        futureExpiredTransactionIds: futureExpiredTransactionIds,
        pastExpiredTransactionIds: pastExpiredTransactionIds,
      };
    });

    expect(result.futureExpiredTransactionIds.length).toEqual(1);
    expect(result.futureExpiredTransactionIds).toContain(
      uncommittedTransactionId
    );
    expect(result.pastExpiredTransactionIds.length).toEqual(0);
    expect(result.allTransactionIds.length).toEqual(2);
    expect(result.allTransactionIds).toContain(committedTransactionId);
    expect(result.allTransactionIds).toContain(uncommittedTransactionId);
  });
});

// COMPILE_TX_SCRIPT TESTS
// =======================================================================================================

interface CompileTxScriptResult {
  scriptRoot: string;
}

export const compileTxScript = async (
  page: Page,
  script: string
): Promise<CompileTxScriptResult> => {
  return await page.evaluate(async (_script: string) => {
    const client = window.client;

    let walletAccount = await client.newWallet(
      window.AccountStorageMode.private(),
      true
    );

    const compiledScript = await client.compileTxScript(_script);

    return {
      scriptRoot: compiledScript.root().toHex(),
    };
  }, script);
};

test.describe("compile_tx_script tests", () => {
  test("compile_tx_script compiles script successfully", async ({ page }) => {
    const script = `
            use.miden::contracts::auth::basic->auth_tx
            use.miden::kernels::tx::prologue
            use.miden::kernels::tx::memory

            begin
                push.0 push.0
                # => [0, 0]
                assert_eq
            end
        `;
    const result = await compileTxScript(page, script);

    expect(result.scriptRoot.length).toBeGreaterThan(1);
  });

  test("compile_tx_script does not compile script successfully", async ({
    page,
  }) => {
    const script = "fakeScript";

    await expect(compileTxScript(page, script)).rejects.toThrow(
      /failed to compile transaction script:/
    );
  });
});
