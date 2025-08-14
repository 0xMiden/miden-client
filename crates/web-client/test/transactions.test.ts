import { expect } from "chai";
import { testingPage } from "./mocha.global.setup.mjs";
import {
  consumeTransaction,
  mintAndConsumeTransaction,
  mintTransaction,
  setupWalletAndFaucet,
} from "./webClientTestUtils";

// GET_TRANSACTIONS TESTS
// =======================================================================================================

interface GetAllTransactionsResult {
  transactionIds: string[];
  uncommittedTransactionIds: string[];
}

const getAllTransactions = async (): Promise<GetAllTransactionsResult> => {
  return await testingPage.evaluate(async () => {
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

describe("get_transactions tests", () => {
  it("get_transactions retrieves all transactions successfully", async () => {
    const { accountId, faucetId } = await setupWalletAndFaucet();

    const { mintResult, consumeResult } = await mintAndConsumeTransaction(
      accountId,
      faucetId
    );

    const result = await getAllTransactions();

    expect(result.transactionIds).to.include(mintResult.transactionId);
    expect(result.transactionIds).to.include(consumeResult.transactionId);
    expect(result.uncommittedTransactionIds.length).to.equal(0);
  });

  it("get_transactions retrieves uncommitted transactions successfully", async () => {
    const { accountId, faucetId } = await setupWalletAndFaucet();
    const { mintResult, consumeResult } = await mintAndConsumeTransaction(
      accountId,
      faucetId
    );
    const { transactionId: uncommittedTransactionId } = await mintTransaction(
      accountId,
      faucetId,
      false,
      false
    );

    const result = await getAllTransactions();

    expect(result.transactionIds).to.include(mintResult.transactionId);
    expect(result.transactionIds).to.include(consumeResult.transactionId);
    expect(result.transactionIds).to.include(uncommittedTransactionId);
    expect(result.transactionIds.length).to.equal(3);

    expect(result.uncommittedTransactionIds).to.include(
      uncommittedTransactionId
    );
    expect(result.uncommittedTransactionIds.length).to.equal(1);
  });

  it("get_transactions retrieves no transactions successfully", async () => {
    const result = await getAllTransactions();

    expect(result.transactionIds.length).to.equal(0);
    expect(result.uncommittedTransactionIds.length).to.equal(0);
  });

  it("get_transactions filters by specific transaction IDs successfully", async () => {
    const { accountId, faucetId } = await setupWalletAndFaucet();

    await mintAndConsumeTransaction(accountId, faucetId);

    const result = await testingPage.evaluate(async () => {
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

    expect(result.allTransactionsCount).to.equal(2);
    expect(result.filteredTransactionIds.length).to.equal(1);
    expect(result.filteredTransactionIds).to.include(
      result.originalTransactionId
    );
  });

  it("get_transactions filters expired transactions successfully", async () => {
    const { accountId, faucetId } = await setupWalletAndFaucet();

    const { transactionId: committedTransactionId } = await mintTransaction(
      accountId,
      faucetId
    );

    const { transactionId: uncommittedTransactionId } = await mintTransaction(
      accountId,
      faucetId,
      false,
      false
    );

    const result = await testingPage.evaluate(async () => {
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

    expect(result.futureExpiredTransactionIds.length).to.equal(1);
    expect(result.futureExpiredTransactionIds).to.include(
      uncommittedTransactionId
    );
    expect(result.pastExpiredTransactionIds.length).to.equal(0);
    expect(result.allTransactionIds.length).to.equal(2);
    expect(result.allTransactionIds).to.include(committedTransactionId);
    expect(result.allTransactionIds).to.include(uncommittedTransactionId);
  });
});

// COMPILE_TX_SCRIPT TESTS
// =======================================================================================================

interface CompileTxScriptResult {
  scriptRoot: string;
}

export const compileTxScript = async (
  script: string
): Promise<CompileTxScriptResult> => {
  return await testingPage.evaluate(async (_script) => {
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

describe("compile_tx_script tests", () => {
  it("compile_tx_script compiles script successfully", async () => {
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
    const result = await compileTxScript(script);

    expect(result.scriptRoot).to.not.be.empty;
  });

  it("compile_tx_script does not compile script successfully", async () => {
    const script = "fakeScript";

    await expect(compileTxScript(script)).to.be.rejectedWith(
      /failed to compile transaction script:/
    );
  });
});
